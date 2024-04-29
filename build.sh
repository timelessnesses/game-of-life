#NOTE: Dont't forget to modify these vars to your setup
JNI_LIBS=./android-project/app/src/main/jniLibs
LIB_NAME=rust_game_of_life.so
SDL2_LIBS=D:/SDL-release-2.30.2/libs
BUILD_MODE=
ANDROID_NDK_HOME=D:/android-ndk-r26d

#copy sdl2 libs into rusts build dir
mkdir -p ./target/aarch64-linux-android/$BUILD_MODE/deps/
mkdir -p ./target/armv7-linux-androideabi/$BUILD_MODE/deps/
mkdir -p ./target/i686-linux-android/$BUILD_MODE/deps/
cp -a $SDL2_LIBS/arm64-v8a/. target/aarch64-linux-android/$BUILD_MODE/deps/
cp -a $SDL2_LIBS/armeabi-v7a/. target/armv7-linux-androideabi/$BUILD_MODE/deps/
cp -a $SDL2_LIBS/x86/. ./target/i686-linux-android/$BUILD_MODE/deps/

#build the libraries
ANDROID_NDK_HOME=$ANDROID_NDK_HOME ANDROID_NDK=$ANDROID_NDK_HOME cargo build --target aarch64-linux-android --$BUILD_MODE
ANDROID_NDK_HOME=$ANDROID_NDK_HOME ANDROID_NDK=$ANDROID_NDK_HOME cargo build --target armv7-linux-androideabi --$BUILD_MODE

#prepare folders...
rm -rf $JNI_LIBS
mkdir $JNI_LIBS
mkdir $JNI_LIBS/arm64-v8a
mkdir $JNI_LIBS/armeabi-v7a
mkdir $JNI_LIBS/x86

#..and copy the rust library into the android studio project, ready for beeing included into the APK
cp target/aarch64-linux-android/$BUILD_MODE/$LIB_NAME $JNI_LIBS/arm64-v8a/libmain.so
cp target/armv7-linux-androideabi/$BUILD_MODE/$LIB_NAME $JNI_LIBS/armeabi-v7a/libmain.so
cp target/i686-linux-android/$BUILD_MODE/$LIB_NAME $JNI_LIBS/x86/libmain.so