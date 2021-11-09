# VCX Pod for iOS

#### Prerequisites

* Minimum supported versions
    * Devices 
      * 64 Bit devices only
      * No support for 32 bit devices
    * OS
      * iOS 10+

* Development environment:
    * Xcode
    * Installed CocoaPods Gem for Ruby. [Install here](https://cocoapods.org)

* **Currently recommended versions**
    * `vcx 0.0.213` for phones
    * `vcx 0.0.214` for simulators


#### Add dependency libraries

1. Verify that you have CocoaPods. If you do not, run `sudo gem install cocoapods`

2. Add the next source to the top of your `Podfile`:

      ```ruby
        source 'https://cdn.cocoapods.org/'
        source 'git@gitlab.com:evernym/mobile/mobile-sdk.git'
      ```

3. Add `pod 'vcx', '0.0.213'` in Podfile inside `target <ProjectName>`

    * `vcx 0.0.213` should be used to run applications on phones
    * `vcx 0.0.214` should be used to run applications on simulators

4. Run `pod install`
