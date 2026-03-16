# Update KosherJava

Pull the latest KosherJava code into the test/java directory

```
git subtree pull --prefix=test/java  https://github.com/KosherJava/zmanim <branch-name> --squash
```

# Pre-requisites

```
# Cargo Prerequisites
cargo install cargo-expand
cargo install flutter_rust_bridge_codegen
# Dart Prerequisites

dart pub get
dart run jni:setup
# Java Prerequisites
# Ensure JAVA_HOME is set, Maven is installed, and that `JAVA_HOME\bin\server` is in the PATH
```

# Generate the Rust Bindings code

```
flutter_rust_bridge_codegen generate
```

# Generate the Java Bindings code

```
dart run jnigen.dart
```

# Build the KosherJava & Rust libraries

```
dart run build.dart
```

# Run the tests

```
dart test
```
