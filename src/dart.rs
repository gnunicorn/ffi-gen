use crate::import::{Import, Instr};
use crate::{Abi, AbiFunction, AbiObject, AbiType, FunctionType, Interface, NumType, Return, Var};
use genco::prelude::*;
use genco::tokens::static_literal;
use heck::*;

pub struct DartGenerator {
    abi: Abi,
    library_name: String,
    cdylib_name: String,
}

impl DartGenerator {
    pub fn new(library_name: String, cdylib_name: String) -> Self {
        Self {
            abi: Abi::native(),
            library_name,
            cdylib_name,
        }
    }

    pub fn generate(&self, iface: Interface) -> dart::Tokens {
        quote! {
            #(static_literal("//")) AUTO GENERATED FILE, DO NOT EDIT.
            #(static_literal("//"))
            #(static_literal("//")) Generated by "ffi-gen".

            #(self.generate_doc(&iface.doc))
            library #(&self.library_name);

            import "dart:async";
            import "dart:convert";
            import "dart:ffi" as ffi;
            import "dart:io" show Platform;
            import "dart:isolate";
            import "dart:typed_data";

            class _DartApiEntry extends ffi.Struct {
                external ffi.Pointer<ffi.Uint8> name;
                external ffi.Pointer<ffi.Void> ptr;
            }

            class _DartApi extends ffi.Struct {
                @ffi.Int32()
                external int major;

                @ffi.Int32()
                external int minor;

                external ffi.Pointer<_DartApiEntry> functions;
            }

            ffi.Pointer<T> _lookupDartSymbol<T extends ffi.NativeType>(String symbol) {
                final ffi.Pointer<_DartApi> api = ffi.NativeApi.initializeApiDLData.cast();
                final ffi.Pointer<_DartApiEntry> functions = api.ref.functions;
                for (var i = 0; i < 100; i++) {
                    final func = functions.elementAt(i).ref;
                    var symbol2 = "";
                    var j = 0;
                    while (func.name.elementAt(j).value != 0) {
                        symbol2 += String.fromCharCode(func.name.elementAt(j).value);
                        j += 1;
                    }
                    if (symbol == symbol2) {
                        return func.ptr.cast();
                    }
                }
                throw "symbol not found";
            }

            class _Box {
                final Api _api;
                final ffi.Pointer<ffi.Void> _ptr;
                final String _dropSymbol;
                bool _dropped;
                bool _moved;
                ffi.Pointer<ffi.Void> _finalizer = ffi.Pointer.fromAddress(0);

                _Box(this._api, this._ptr, this._dropSymbol) : _dropped = false, _moved = false;

                late final _dropPtr = _api._lookup<
                    ffi.NativeFunction<
                        ffi.Void Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>)>>(_dropSymbol);

                late final _drop = _dropPtr.asFunction<
                    void Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>)>();

                int borrow() {
                    if (_dropped) {
                        throw StateError("use after free");
                    }
                    if (_moved) {
                        throw StateError("use after move");
                    }
                    return _ptr.address;
                }

                int move() {
                    if (_dropped) {
                        throw StateError("use after free");
                    }
                    if (_moved) {
                        throw StateError("can't move value twice");
                    }
                    _moved = true;
                    _api._unregisterFinalizer(this);
                    return _ptr.address;
                }

                void drop() {
                    if (_dropped) {
                        throw StateError("double free");
                    }
                    if (_moved) {
                        throw StateError("can't drop moved value");
                    }
                    _dropped = true;
                    _api._unregisterFinalizer(this);
                    _drop(ffi.Pointer.fromAddress(0), _ptr);
                }
            }

            #(for ty in "Int8 Uint8 Int16 Uint16 Int32 Uint32 Int64 Uint64 Float32 Float64".split(' ') => #(self.generate_ffi_buffer(ty)))

            #(static_literal("///")) Implements Iterable and Iterator for a rust iterator.
            class Iter<T> extends Iterable<T> implements Iterator<T> {
                final _Box _box;
                final T? Function(int) _next;

                Iter._(this._box, this._next);

                @override
                Iterator<T> get iterator => this;

                T? _current;

                @override
                T get current => _current!;

                @override
                bool moveNext() {
                    final next = _next(_box.borrow());
                    if (next == null) {
                        return false;
                    } else {
                        _current = next;
                        return true;
                    }
                }

                void drop() {
                    _box.drop();
                }
            }

            abstract class CustomIterable<T> {
              int get length;
              T elementAt(int index);
            }

            class CustomIterator<T, U extends CustomIterable<T>> implements Iterator<T> {
              final U _iterable;
              int _currentIndex = -1;

              CustomIterator(this._iterable);

              @override
              T get current => _iterable.elementAt(_currentIndex);

              @override
              bool moveNext() {
                _currentIndex++;
                return _currentIndex < _iterable.length;
              }
            }

            Future<T> _nativeFuture<T>(_Box box, T? Function(int, int, int) nativePoll) {
                final completer = Completer<T>();
                final rx = ReceivePort();
                void poll() {
                    try {
                        final ret = nativePoll(box.borrow(), ffi.NativeApi.postCObject.address, rx.sendPort.nativePort);
                        if (ret == null) {
                            return;
                        }
                        completer.complete(ret);
                    } catch(err) {
                        completer.completeError(err);
                    }
                    rx.close();
                    box.drop();
                }
                rx.listen((dynamic _message) => poll());
                poll();
                return completer.future;
            }

            Stream<T> _nativeStream<T>(_Box box, T? Function(int, int, int, int) nativePoll) {
                final controller = StreamController<T>();
                final rx = ReceivePort();
                final done = ReceivePort();
                void poll() {
                    try {
                        final ret = nativePoll(
                            box.borrow(),
                            ffi.NativeApi.postCObject.address,
                            rx.sendPort.nativePort,
                            done.sendPort.nativePort,
                        );
                        if (ret != null) {
                            controller.add(ret);
                        }
                    } catch(err) {
                        controller.addError(err);
                    }
                }
                void close() {
                    rx.close();
                    done.close();
                    box.drop();
                }
                controller.onCancel = close;
                rx.listen((dynamic _message) => poll());
                done.listen((dynamic _message) => controller.close());
                poll();
                return controller.stream;
            }

            #(static_literal("///")) Main entry point to library.
            class Api {
                #(static_literal("///")) Holds the symbol lookup function.
                final ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName)
                    _lookup;

                #(static_literal("///")) The symbols are looked up in [dynamicLibrary].
                Api(ffi.DynamicLibrary dynamicLibrary)
                    : _lookup = dynamicLibrary.lookup;

                #(static_literal("///")) The symbols are looked up with [lookup].
                Api.fromLookup(
                    ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName)
                        lookup)
                    : _lookup = lookup;

                #(static_literal("///")) The library is loaded from the executable.
                factory Api.loadStatic() {
                    return Api(ffi.DynamicLibrary.executable());
                }

                #(static_literal("///")) The library is dynamically loaded.
                factory Api.loadDynamic(String name) {
                    return Api(ffi.DynamicLibrary.open(name));
                }

                #(static_literal("///")) The library is loaded based on platform conventions.
                factory Api.load() {
                    String? name;
                    if (Platform.isLinux) name = #_(#("lib")#(&self.cdylib_name)#(".so"));
                    if (Platform.isAndroid) name = #_(#("lib")#(&self.cdylib_name)#(".so"));
                    if (Platform.isMacOS) name = #_(#("lib")#(&self.cdylib_name)#(".dylib"));
                    if (Platform.isIOS) name = "";
                    if (Platform.isWindows) name = #_(#(&self.cdylib_name)#(".dll"));
                    if (name == null) {
                        throw UnsupportedError(#_("This platform is not supported."));
                    }
                    if (name == "") {
                        return Api.loadStatic();
                    } else {
                        return Api.loadDynamic(name);
                    }
                }

                late final _registerPtr = _lookupDartSymbol<
                    ffi.NativeFunction<ffi.Pointer<ffi.Void> Function(
                        ffi.Handle, ffi.Pointer<ffi.Void>, ffi.IntPtr, ffi.Pointer<ffi.Void>)>>("Dart_NewFinalizableHandle");

                late final _register = _registerPtr.asFunction<
                    ffi.Pointer<ffi.Void> Function(Object, ffi.Pointer<ffi.Void>, int, ffi.Pointer<ffi.Void>)>();

                ffi.Pointer<ffi.Void> _registerFinalizer(_Box boxed) {
                    return _register(boxed, boxed._ptr, 42, boxed._dropPtr.cast());
                }

                late final _unregisterPtr = _lookupDartSymbol<
                    ffi.NativeFunction<ffi.Void Function(ffi.Pointer<ffi.Void>, ffi.Handle)>>("Dart_DeleteFinalizableHandle");

                late final _unregister = _unregisterPtr.asFunction<void Function(ffi.Pointer<ffi.Void>, _Box)>();

                void _unregisterFinalizer(_Box boxed) {
                    _unregister(boxed._finalizer, boxed);
                }

                ffi.Pointer<T> __allocate<T extends ffi.NativeType>(int byteCount, int alignment) {
                    return _allocate(byteCount, alignment).cast();
                }

                void __deallocate<T extends ffi.NativeType>(ffi.Pointer pointer, int byteCount, int alignment) {
                    _deallocate(pointer.cast(), byteCount, alignment);
                }

                #(for func in iface.functions() => #(self.generate_function(&func)))

                late final _allocatePtr = _lookup<
                    ffi.NativeFunction<
                        ffi.Pointer<ffi.Uint8> Function(ffi.IntPtr, ffi.IntPtr)>>("allocate");

                late final _allocate = _allocatePtr.asFunction<
                    ffi.Pointer<ffi.Uint8> Function(int, int)>();

                late final _deallocatePtr = _lookup<
                    ffi.NativeFunction<
                        ffi.Void Function(ffi.Pointer<ffi.Uint8>, ffi.IntPtr, ffi.IntPtr)>>("deallocate");

                late final _deallocate = _deallocatePtr.asFunction<
                    void Function(ffi.Pointer<ffi.Uint8>, int, int)>();

                late final _ffiBufferAddressPtr = _lookup<
                    ffi.NativeFunction<
                        ffi.Pointer<ffi.Uint8> Function(ffi.IntPtr)>>("__ffi_buffer_address");

                late final _ffiBufferAddress = _ffiBufferAddressPtr.asFunction<
                    ffi.Pointer<ffi.Uint8> Function(int)>();

                late final _ffiBufferSizePtr = _lookup<
                    ffi.NativeFunction<
                        ffi.Uint32 Function(ffi.IntPtr)>>("__ffi_buffer_size");

                late final _ffiBufferSize = _ffiBufferSizePtr.asFunction<
                    int Function(int)>();

                #(for iter in iface.iterators() => #(self.generate_function(&iter.next())))
                #(for fut in iface.futures() => #(self.generate_function(&fut.poll())))
                #(for stream in iface.streams() => #(self.generate_function(&stream.poll())))

                #(for func in iface.imports(&self.abi) => #(self.generate_wrapper(func)))
                #(for ty in iface.listed_types() => #(self.generate_list_methods(ty.as_str())))
            }

            #(for obj in iface.objects() => #(self.generate_object(obj)))

            #(for func in iface.imports(&self.abi) => #(self.generate_return_struct(&func.ffi_ret)))

            #(for ty in iface.listed_types() => #(self.generate_list_type(ty.as_str())))
        }
    }

    fn generate_list_methods(&self, ty: &str) -> dart::Tokens {
        let list_name_s = format!("FfiList{}", ty);
        let list_name = list_name_s.as_str();
        quote!(

            #list_name #(format!("create{}", list_name))() {
                final ffi.Pointer<ffi.Void> list_ptr = ffi.Pointer.fromAddress(#(format!("_ffiList{}Create", ty))());
                final list_box = _Box(this, list_ptr, #(format!("\"drop_box_{}\"", list_name)));
                return #list_name._(this, list_box);
            }

            late final #(format!("_ffiList{}CreatePtr", ty)) = _lookup<
                ffi.NativeFunction<
                    ffi.IntPtr Function()>>(#(format!("\"__{}Create\"", list_name)));

            late final #(format!("_ffiList{}Create", ty)) = #(format!("_ffiList{}CreatePtr", ty)).asFunction<
                int Function()>();

            late final #(format!("_ffiList{}LenPtr", ty)) = _lookup<
                ffi.NativeFunction<
                    ffi.Uint32 Function(ffi.IntPtr)>>(#(format!("\"__{}Len\"", list_name)));

            late final #(format!("_ffiList{}Len", ty)) = #(format!("_ffiList{}LenPtr", ty)).asFunction<
                int Function(int)>();

            late final #(format!("_ffiList{}ElementAtPtr", ty)) = _lookup<
                ffi.NativeFunction<
                    ffi.IntPtr Function(ffi.IntPtr, ffi.Uint32)>>(#(format!("\"__{}ElementAt\"", list_name)));

            late final #(format!("_ffiList{}ElementAt", ty)) = #(format!("_ffiList{}ElementAtPtr", ty)).asFunction<
                int Function(int, int)>();
        )
    }

    fn generate_list_type(&self, ty: &str) -> dart::Tokens {
        let list_name_s = format!("FfiList{}", ty);
        let list_name = list_name_s.as_str();
        quote!(
            class #list_name extends Iterable<#ty> implements CustomIterable<#ty> {
                final Api _api;
                final _Box _box;

                #list_name._(this._api, this._box);

                @override
                Iterator<CustomType> get iterator => CustomIterator(this);

                @override
                int get length {
                    return _api.#(format!("_ffiList{}Len", ty))(_box.borrow());
                }

                @override
                #ty elementAt(int index) {
                    final address = _api.#(format!("_ffiList{}ElementAt", ty))(_box.borrow(), index);
                    final reference = _Box(_api, ffi.Pointer.fromAddress(address), "drop_box_Leak");
                    return #ty._(_api, reference);
                }

                void drop() {
                  _box.drop();
                }
            }
        )
    }

    fn generate_object(&self, obj: AbiObject) -> dart::Tokens {
        quote! {
            #(self.generate_doc(&obj.doc))
            class #(&obj.name) {
                final Api _api;
                final _Box _box;

                #(&obj.name)._(this._api, this._box);

                #(for func in &obj.methods => #(self.generate_function(func)))

                #(static_literal("///")) Manually drops the object and unregisters the FinalizableHandle.
                void drop() {
                    _box.drop();
                }
            }
        }
    }

    fn generate_function(&self, func: &AbiFunction) -> dart::Tokens {
        let ffi = self.abi.import(func);
        let api = match &func.ty {
            FunctionType::Constructor(_) => "api",
            FunctionType::Method(_) => "_api",
            FunctionType::Function
            | FunctionType::NextIter(_, _)
            | FunctionType::PollFuture(_, _)
            | FunctionType::PollStream(_, _) => "this",
        };
        let name = match &func.ty {
            FunctionType::NextIter(_, _)
            | FunctionType::PollFuture(_, _)
            | FunctionType::PollStream(_, _) => {
                format!("__{}", self.ident(&ffi.symbol))
            }
            _ => self.ident(&func.name),
        };
        let args = quote!(#(for (name, ty) in &ffi.abi_args => #(self.generate_type(ty)) #(self.ident(name)),));
        let body = quote!(#(for instr in &ffi.instr => #(self.generate_instr(api, instr))));
        let ret = if let Some(ret) = ffi.abi_ret.as_ref() {
            self.generate_type(ret)
        } else {
            quote!(void)
        };
        let doc = self.generate_doc(&func.doc);
        match &func.ty {
            FunctionType::Constructor(_object) => quote! {
                #doc
                static #ret #name(Api api, #args) {
                    #body
                }
            },
            _ => {
                quote! {
                    #doc
                    #ret #name(#args) {
                        #body
                    }
                }
            }
        }
    }

    fn generate_ffi_buffer(&self, ty: &str) -> dart::Tokens {
        let bytes = ty.chars().skip_while(|&c| !c.is_digit(10)).collect::<String>().parse::<u32>().unwrap() / 8;
        let pointer_name = match ty {
            "Float32" => "Float",
            "Float64" => "Double",
            _ => ty,
        };
        let class_name = format!("FfiBuffer{}", ty);
        let method_name = format!("to{}List", ty);
        quote! {
            class #(&class_name) {
                final Api _api;
                final _Box _box;

                #(&class_name)._(this._api, this._box);

                void drop() {
                    _box.drop();
                }

                #(ty)List #(method_name)() {
                    final buffer = _box.borrow();
                    final addressRaw = _api._ffiBufferAddress(buffer).address;
                    final size = _api._ffiBufferSize(buffer) ~/ #(bytes);
                    return ffi.Pointer<ffi.#(pointer_name)>.fromAddress(addressRaw).asTypedList(size);
                }
            }
        }
    }

    fn generate_instr(&self, api: &str, instr: &Instr) -> dart::Tokens {
        match instr {
            Instr::BorrowSelf(out) => quote!(#(self.var(out)) = _box.borrow();),
            Instr::BorrowObject(in_, out)
            | Instr::BorrowIter(in_, out)
            | Instr::BorrowFuture(in_, out)
            | Instr::BorrowStream(in_, out) => {
                quote!(#(self.var(out)) = #(self.var(in_))._box.borrow();)
            }
            Instr::MoveObject(in_, out)
            | Instr::MoveIter(in_, out)
            | Instr::MoveFuture(in_, out)
            | Instr::MoveStream(in_, out) => {
                quote!(#(self.var(out)) = #(self.var(in_))._box.move();)
            }
            Instr::LiftObject(obj, box_, drop, out) => quote! {
                final ffi.Pointer<ffi.Void> #(self.var(box_))_0 = ffi.Pointer.fromAddress(#(self.var(box_)));
                final #(self.var(box_))_1 = _Box(#api, #(self.var(box_))_0, #_(#drop));
                #(self.var(box_))_1._finalizer = #api._registerFinalizer(#(self.var(box_))_1);
                final #(self.var(out)) = #obj._(#api, #(self.var(box_))_1);
            },
            Instr::BindArg(arg, out) => quote!(final #(self.var(out)) = #(self.ident(arg));),
            Instr::BindRets(ret, vars) => match vars.len() {
                0 => quote!(),
                1 => quote!(final #(self.var(&vars[0])) = #(self.var(ret));),
                _ => quote! {
                    #(for (idx, var) in vars.iter().enumerate() =>
                        final #(self.var(var)) = #(self.var(ret)).#(format!("arg{}", idx));)
                },
            },
            Instr::LowerNum(in_, out, _num) => {
                quote!(#(self.var(out)) = #(self.var(in_));)
            }
            Instr::LiftNum(in_, out, _num) => {
                quote!(final #(self.var(out)) = #(self.var(in_));)
            }
            Instr::LowerBool(in_, out) => {
                quote!(#(self.var(out)) = #(self.var(in_)) ? 1 : 0;)
            }
            Instr::LiftBool(in_, out) => {
                quote!(final #(self.var(out)) = #(self.var(in_)) > 0;)
            }
            Instr::Deallocate(ptr, len, size, align) => quote! {
                if (#(self.var(len)) > 0) {
                    final ffi.Pointer<ffi.Void> #(self.var(ptr))_0;
                    #(self.var(ptr))_0 = ffi.Pointer.fromAddress(#(self.var(ptr)));
                    #api.__deallocate(#(self.var(ptr))_0, #(self.var(len)) * #(*size), #(*align));
                }
            },
            Instr::LowerString(in_, ptr, len, cap, size, align) => quote! {
                final #(self.var(in_))_0 = utf8.encode(#(self.var(in_)));
                #(self.var(len)) = #(self.var(in_))_0.length;
                final ffi.Pointer<ffi.Uint8> #(self.var(ptr))_0 =
                    #api.__allocate(#(self.var(len)) * #(*size), #(*align));
                final Uint8List #(self.var(ptr))_1 = #(self.var(ptr))_0.asTypedList(#(self.var(len)));
                #(self.var(ptr))_1.setAll(0, #(self.var(in_))_0);
                #(self.var(ptr)) = #(self.var(ptr))_0.address;
                #(self.var(cap)) = #(self.var(len));
            },
            Instr::LiftString(ptr, len, out) => quote! {
                final ffi.Pointer<ffi.Uint8> #(self.var(ptr))_0 = ffi.Pointer.fromAddress(#(self.var(ptr)));
                final #(self.var(out)) = utf8.decode(#(self.var(ptr))_0.asTypedList(#(self.var(len))));
            },
            Instr::LowerVec(in_, ptr, len, cap, ty, size, align) => quote! {
                #(self.var(len)) = #(self.var(in_)).length;
                final ffi.Pointer<#(self.generate_native_num_type(*ty))> #(self.var(ptr))_0 =
                    #api.__allocate(#(self.var(len)) * #(*size), #(*align));
                final #(self.var(ptr))_1 = #(self.var(ptr))_0.asTypedList(#(self.var(len)));
                #(self.var(ptr))_1.setAll(0, #(self.var(in_)));
                #(self.var(ptr)) = #(self.var(ptr))_0.address;
                #(self.var(cap)) = #(self.var(len));
            },
            Instr::LiftVec(ptr, len, out, ty) => quote! {
                final ffi.Pointer<#(self.generate_native_num_type(*ty))> #(self.var(ptr))_0 =
                    ffi.Pointer.fromAddress(#(self.var(ptr)));
                final #(self.var(out)) = #(self.var(ptr))_0.asTypedList(#(self.var(len))).toList();
            },
            Instr::Call(symbol, ret, args) => {
                let api = if api == "this" {
                    quote!()
                } else {
                    quote!(#api.)
                };
                let invoke = quote!(#(api)#(format!("_{}", self.ident(symbol)))(#(for arg in args => #(self.var(arg)),)););
                if let Some(ret) = ret {
                    quote!(final #(self.var(ret)) = #invoke)
                } else {
                    invoke
                }
            }
            Instr::DefineArgs(vars) => quote! {
                #(for var in vars => var #(self.var(var)) = #(self.literal(var.ty.num()));)
            },
            Instr::ReturnValue(ret) => quote!(return #(self.var(ret));),
            Instr::ReturnVoid => quote!(return;),
            Instr::HandleNull(var) => quote! {
                if (#(self.var(var)) == 0) {
                    return null;
                }
            },
            Instr::LowerOption(arg, var, some, some_instr) => quote! {
                if (#(self.var(arg)) == null) {
                    #(self.var(var)) = 0;
                } else {
                    #(self.var(var)) = 1;
                    final #(self.var(some)) = #(self.var(arg));
                    #(for inst in some_instr => #(self.generate_instr(api, inst)))
                }
            },
            Instr::HandleError(var, ptr, len, cap) => quote! {
                if (#(self.var(var)) == 0) {
                    final ffi.Pointer<ffi.Uint8> #(self.var(ptr))_0 = ffi.Pointer.fromAddress(#(self.var(ptr)));
                    final #(self.var(var))_0 = utf8.decode(#(self.var(ptr))_0.asTypedList(#(self.var(len))));
                    if (#(self.var(len)) > 0) {
                        final ffi.Pointer<ffi.Void> #(self.var(ptr))_0;
                        #(self.var(ptr))_0 = ffi.Pointer.fromAddress(#(self.var(ptr)));
                        #api.__deallocate(#(self.var(ptr))_0, #(self.var(cap)), 1);
                    }
                    throw #(self.var(var))_0;
                }
            },
            Instr::LiftIter(box_, next, drop, out) => quote! {
                final ffi.Pointer<ffi.Void> #(self.var(box_))_0 = ffi.Pointer.fromAddress(#(self.var(box_)));
                final #(self.var(box_))_1 = _Box(#api, #(self.var(box_))_0, #_(#drop));
                #(self.var(box_))_1._finalizer = #api._registerFinalizer(#(self.var(box_))_1);
                final #(self.var(out)) = Iter._(#(self.var(box_))_1, #api.#(format!("__{}", self.ident(next))));
            },
            Instr::LiftFuture(box_, poll, drop, out) => quote! {
                final ffi.Pointer<ffi.Void> #(self.var(box_))_0 = ffi.Pointer.fromAddress(#(self.var(box_)));
                final #(self.var(box_))_1 = _Box(#api, #(self.var(box_))_0, #_(#drop));
                #(self.var(box_))_1._finalizer = #api._registerFinalizer(#(self.var(box_))_1);
                final #(self.var(out)) = _nativeFuture(#(self.var(box_))_1, #api.#(format!("__{}", self.ident(poll))));
            },
            Instr::LiftStream(box_, poll, drop, out) => quote! {
                final ffi.Pointer<ffi.Void> #(self.var(box_))_0 = ffi.Pointer.fromAddress(#(self.var(box_)));
                final #(self.var(box_))_1 = _Box(#api, #(self.var(box_))_0, #_(#drop));
                #(self.var(box_))_1._finalizer = #api._registerFinalizer(#(self.var(box_))_1);
                final #(self.var(out)) = _nativeStream(#(self.var(box_))_1, #api.#(format!("__{}", self.ident(poll))));
            },
            Instr::LiftTuple(vars, out) => match vars.len() {
                0 => quote!(),
                1 => quote!(final #(self.var(out)) = #(self.var(&vars[0]));),
                _ => quote! {
                    final List #(self.var(out)) = [];
                    #(for var in vars => #(self.var(out)).add(#(self.var(var)));)
                },
            },
            Instr::LiftNumFromU32Tuple(..) | Instr::LowerNumFromU32Tuple(..) => unreachable!(),
        }
    }

    fn var(&self, var: &Var) -> dart::Tokens {
        quote!(#(format!("tmp{}", var.binding)))
    }

    fn literal(&self, ty: NumType) -> dart::Tokens {
        match ty {
            NumType::F32 | NumType::F64 => quote!(0.0),
            _ => quote!(0),
        }
    }

    fn generate_wrapper(&self, func: Import) -> dart::Tokens {
        let native_args =
            quote!(#(for var in &func.ffi_args => #(self.generate_native_num_type(var.ty.num())),));
        let wrapped_args = quote!(#(for var in &func.ffi_args => #(self.generate_wrapped_num_type(var.ty.num())),));
        let native_ret = self.generate_native_return_type(&func.ffi_ret);
        let wrapped_ret = self.generate_wrapped_return_type(&func.ffi_ret);
        let symbol_ptr = format!("_{}Ptr", self.ident(&func.symbol));
        quote! {
            late final #(&symbol_ptr) =
                _lookup<ffi.NativeFunction<#native_ret Function(#native_args)>>(#_(#(&func.symbol)));

            late final #(format!("_{}", self.ident(&func.symbol))) =
                #symbol_ptr.asFunction<#wrapped_ret Function(#wrapped_args)>();
        }
    }

    fn generate_type(&self, ty: &AbiType) -> dart::Tokens {
        match ty {
            AbiType::Num(ty) => self.generate_wrapped_num_type(*ty),
            AbiType::Isize | AbiType::Usize => quote!(int),
            AbiType::Bool => quote!(bool),
            AbiType::RefStr | AbiType::String => quote!(String),
            AbiType::RefSlice(ty) | AbiType::Vec(ty) => {
                quote!(List<#(self.generate_wrapped_num_type(*ty))>)
            }
            AbiType::Option(ty) => quote!(#(self.generate_type(&**ty))?),
            AbiType::Result(ty) => self.generate_type(&**ty),
            AbiType::Tuple(tuple) => match tuple.len() {
                0 => quote!(void),
                1 => self.generate_type(&tuple[0]),
                _ => quote!(List<dynamic>),
            },
            AbiType::RefObject(ty) | AbiType::Object(ty) => quote!(#ty),
            AbiType::RefIter(ty) | AbiType::Iter(ty) => quote!(Iter<#(self.generate_type(ty))>),
            AbiType::RefFuture(ty) | AbiType::Future(ty) => {
                quote!(Future<#(self.generate_type(ty))>)
            }
            AbiType::RefStream(ty) | AbiType::Stream(ty) => {
                quote!(Stream<#(self.generate_type(ty))>)
            }
            AbiType::Buffer(ty) => quote!(#(ffi_buffer_name_for(*ty))),
            AbiType::List(ty) => quote!(#(format!("FfiList{}", ty))),
        }
    }

    fn generate_wrapped_num_type(&self, ty: NumType) -> dart::Tokens {
        match ty {
            NumType::F32 | NumType::F64 => quote!(double),
            _ => quote!(int),
        }
    }

    fn generate_native_num_type(&self, ty: NumType) -> dart::Tokens {
        match ty {
            NumType::I8 => quote!(ffi.Int8),
            NumType::I16 => quote!(ffi.Int16),
            NumType::I32 => quote!(ffi.Int32),
            NumType::I64 => quote!(ffi.Int64),
            NumType::U8 => quote!(ffi.Uint8),
            NumType::U16 => quote!(ffi.Uint16),
            NumType::U32 => quote!(ffi.Uint32),
            NumType::U64 => quote!(ffi.Uint64),
            NumType::F32 => quote!(ffi.Float),
            NumType::F64 => quote!(ffi.Double),
        }
    }

    fn generate_native_return_type(&self, ret: &Return) -> dart::Tokens {
        match ret {
            Return::Void => quote!(ffi.Void),
            Return::Num(var) => self.generate_native_num_type(var.ty.num()),
            Return::Struct(_, s) => quote!(#(format!("_{}", self.type_ident(s)))),
        }
    }

    fn generate_wrapped_return_type(&self, ret: &Return) -> dart::Tokens {
        match ret {
            Return::Void => quote!(void),
            Return::Num(var) => self.generate_wrapped_num_type(var.ty.num()),
            Return::Struct(_, s) => quote!(#(format!("_{}", self.type_ident(s)))),
        }
    }

    fn generate_return_struct(&self, ret: &Return) -> dart::Tokens {
        if let Return::Struct(vars, name) = ret {
            quote! {
                class #(format!("_{}", self.type_ident(name))) extends ffi.Struct {
                    #(for (i, var) in vars.iter().enumerate() => #(self.generate_return_struct_field(i, var.ty.num())))
                }
            }
        } else {
            quote!()
        }
    }

    fn generate_return_struct_field(&self, i: usize, ty: NumType) -> dart::Tokens {
        quote! {
            @#(self.generate_native_num_type(ty))()
            external #(self.generate_wrapped_num_type(ty)) #(format!("arg{}", i));
        }
    }

    fn generate_doc(&self, doc: &[String]) -> dart::Tokens {
        quote!(#(for line in doc => #(static_literal("///")) #line #<push>))
    }

    fn type_ident(&self, s: &str) -> String {
        sanitize_identifier(&s.to_upper_camel_case())
    }

    fn ident(&self, s: &str) -> String {
        sanitize_identifier(&s.to_lower_camel_case())
    }
}

fn sanitize_identifier(id: &str) -> String {
    if RESERVED_IDENTIFIERS.contains(&id) {
        format!("{}_", id)
    } else {
        id.to_string()
    }
}

// https://dart.dev/guides/language/language-tour#keywords
static RESERVED_IDENTIFIERS: [&str; 63] = [
    "abstract",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "covariant",
    "default",
    "deferred",
    "do",
    "dynamic",
    "else",
    "enum",
    "export",
    "extends",
    "extension",
    "external",
    "factory",
    "false",
    "final",
    "finally",
    "for",
    "Function",
    "get",
    "hide",
    "if",
    "implements",
    "import",
    "in",
    "interface",
    "is",
    "late",
    "library",
    "mixin",
    "new",
    "null",
    "on",
    "operator",
    "part",
    "required",
    "rethrow",
    "return",
    "set",
    "show",
    "static",
    "super",
    "switch",
    "sync",
    "this",
    "throw",
    "true",
    "try",
    "typedef",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

#[cfg(feature = "test_runner")]
#[doc(hidden)]
pub mod test_runner {
    use super::*;
    use crate::{Abi, RustGenerator};
    use anyhow::Result;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use trybuild::TestCases;

    pub fn compile_pass(iface: &str, rust: rust::Tokens, dart: dart::Tokens) -> Result<()> {
        let iface = Interface::parse(iface)?;
        let mut rust_file = NamedTempFile::new()?;
        let rust_gen = RustGenerator::new(Abi::native());
        let rust_tokens = rust_gen.generate(iface.clone());
        let mut dart_file = NamedTempFile::new()?;
        let dart_gen = DartGenerator::new("compile_pass".to_string(), "compile_pass".to_string());
        let dart_tokens = dart_gen.generate(iface);

        let library_tokens = quote! {
            #rust_tokens
            #rust
        };

        let bin_tokens = quote! {
            #dart_tokens

            extension on List {
                bool equals(List list) {
                    if (this.length != list.length) return false;
                    for (int i = 0; i < this.length; i++) {
                        if (this[i] != list[i]) {
                            return false;
                        }
                    }
                    return true;
                }
            }

            void main() async {
                final api = Api.load();
                #dart
            }
        };

        let library = library_tokens.to_file_string()?;
        rust_file.write_all(library.as_bytes())?;
        let bin = bin_tokens.to_file_string()?;
        dart_file.write_all(bin.as_bytes())?;

        let library_dir = tempfile::tempdir()?;
        let library_file = library_dir.as_ref().join("libcompile_pass.so");
        let runner_tokens: rust::Tokens = quote! {
            fn main() {
                use std::process::Command;
                let ret = Command::new("rustc")
                    .arg("--edition")
                    .arg("2021")
                    .arg("--crate-name")
                    .arg("compile_pass")
                    .arg("--crate-type")
                    .arg("cdylib")
                    .arg("--cfg")
                    .arg("feature=\"test_runner\"")
                    .arg("-o")
                    .arg(#(quoted(library_file.as_path().to_str().unwrap())))
                    .arg(#(quoted(rust_file.as_ref().to_str().unwrap())))
                    .status()
                    .unwrap()
                    .success();
                assert!(ret);
                // println!("{}", #_(#bin));
                let ret = Command::new("dart")
                    .env("LD_LIBRARY_PATH", #(quoted(library_dir.as_ref().to_str().unwrap())))
                    .arg("--enable-asserts")
                    //.arg("--observe")
                    //.arg("--write-service-info=service.json")
                    .arg(#(quoted(dart_file.as_ref().to_str().unwrap())))
                    .status()
                    .unwrap()
                    .success();
                assert!(ret);
            }
        };

        let mut runner_file = NamedTempFile::new()?;
        let runner = runner_tokens.to_file_string()?;
        runner_file.write_all(runner.as_bytes())?;

        let test = TestCases::new();
        test.pass(runner_file.as_ref());
        Ok(())
    }
}

pub fn ffi_buffer_name_for(ty: NumType) -> &'static str {
    match ty {
        NumType::U8 => "FfiBufferUint8",
        NumType::U16 => "FfiBufferUint16",
        NumType::U32 => "FfiBufferUint32",
        NumType::U64 => "FfiBufferUint64",
        NumType::I8 => "FfiBufferInt8",
        NumType::I16 => "FfiBufferInt16",
        NumType::I32 => "FfiBufferInt32",
        NumType::I64 => "FfiBufferInt64",
        NumType::F32 => "FfiBufferFloat32",
        NumType::F64 => "FfiBufferFloat64",
    }
}