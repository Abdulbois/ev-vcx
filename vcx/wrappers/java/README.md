# Vcx SDK for Java and Android

This is a Java wrapper for VCX library. 
VCX is the open-source library on top of Libindy which fully implements the credentials exchange.

**Note**: This library is currently in experimental state.

This Java wrapper currently requires Java 8.

### How to install
In your maven project add to pom.xml file next content:

1. Inside repositories tag block add:
    

    <repository>
        <id>evernym</id>
        <url>https://evernym.mycloudrepo.io/public/repositories/libvcx-java</url>
    </repository>


2. Inside dependencies tag block add:    
    
    
    <dependency>
        <groupId>com.evernym</groupId>
        <artifactId>vcx</artifactId>
        <version>0.12.0.1738</version>
    </dependency>
     
**Note** that before you can use java wrapper you must install  c-callable VDR-Tools and Vcx.  
* See the section "Installing the SDK" in the [VDR Tools documentation](https://gitlab.com/evernym/verity/vdr-tools/-/blob/main/README.md) 
* See the section "Installing VCX" in the [VCX documentation](../../../README.md) 

### How to build

## JAR

 - run `./gradlew clean build`. 

The jar will be present in `ev-vcx/vcx/wrappers/java/vcx/build/libs`

## AAR

 - Copy the binaries i.e `libvcx.so` to folder `ev-vcx/vcx/wrappers/java/vcx/android/src/main/jniLibs/<ARCH>`.
    - Make sure the binaries are in correct architecture folders.
 - run `./gradlew clean build --project-dir=android` in folder `ev-vcx/vcx/wrappers/java/vcx`

###Publishing the AAR
- run `./gradlew clean assemble --project-dir=android` in folder `ev-vcx/vcx/wrappers/java/vcx`

Aar will be present in `ev-vcx/vcx/wrappers/java/vcx/android/build/outputs/aar`

#### Logging
The Java wrapper uses slf4j as a facade for various logging frameworks, such as java.util.logging, logback and log4j.
