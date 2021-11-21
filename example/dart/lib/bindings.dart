// AUTO GENERATED FILE, DO NOT EDIT.
//
// Generated by "ffi-gen".

import "dart:ffi" as ffi;
import "dart:io" show Platform;

class Api {
  /// Holds the symbol lookup function.
  final ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName)
      _lookup;

  /// The symbols are looked up in [dynamicLibrary].
  Api(ffi.DynamicLibrary dynamicLibrary) : _lookup = dynamicLibrary.lookup;

  /// The symbols are looked up with [lookup].
  Api.fromLookup(
      ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName)
          lookup)
      : _lookup = lookup;

  /// The library is loaded from the executable.
  factory Api.loadStatic() {
    return Api(ffi.DynamicLibrary.executable());
  }

  /// The library is dynamically loaded.
  factory Api.loadDynamic(String name) {
    return Api(ffi.DynamicLibrary.open(name));
  }

  /// The library is loaded based on platform conventions.
  factory Api.load() {
    String? name;
    if (Platform.isLinux) name = "libapi.so";
    if (Platform.isAndroid) name = "libapi.so";
    if (Platform.isMacOS) name = "libapi.dylib";
    if (Platform.isIOS) name = "\"\"";
    if (Platform.isWindows) "api.dll";
    if (name == null) {
      throw UnsupportedError("\"This platform is not supported.\"");
    }
    if (name == "") {
      return Api.loadStatic();
    } else {
      return Api.loadDynamic(name);
    }
  }

  void hello_world() {
    _hello_world();
  }

  late final _hello_worldPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function()>>("__hello_world");

  late final _hello_world = _hello_worldPtr.asFunction<void Function()>();
}
