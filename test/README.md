# Pre-requisites

cargo binstall cargo-expand
cargo binstall flutter_rust_bridge_codegen

# Generate the Rust Bindings code

flutter_rust_bridge_codegen generate

# Building the Java Jar

cd kosher-java
mvn -DskipTests -Dmaven.javadoc.skip=true -Dmaven.source.skip=true package

# Generate the Java Bindings code

dart run jni:setup
dart run jnigen.dart

# Test

// On Windows ($env:Path += ";${env:JAVA_HOME}\bin\server")
fvm dart test
