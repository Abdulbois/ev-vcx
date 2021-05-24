VCX_VERSION=$1

OUTPUTDIR=output
DIR=vcx/wrappers/react-native
CURDIR=$(pwd)

cd $DIR

sed -riE "s|com.evernym:vcx:.*@aar|com.evernym:vcx:${VCX_VERSION}@aar|" android/build.gradle

npm i
npm run build
npm pack

rename \s/react-native-vcx-wrapper-/react-native-vcx-wrapper_/ *.tgz

cd $CURDIR
cp $DIR/react-native-vcx-wrapper*.tgz $OUTPUTDIR
