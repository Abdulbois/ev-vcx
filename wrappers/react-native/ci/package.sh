VCX_VERSION=$1

OUTPUTDIR=output
DIR=vcx/wrappers/react-native
CURDIR=$(pwd)

cd $DIR

sed -riE "s|com.evernym:vcx:.*@aar|com.evernym:vcx:${VCX_VERSION}@aar|" android/build.gradle

npm i
npm run build
npm pack

rename \s/evernym-react-native-sdk-/evernym-react-native-sdk_/ *.tgz

cd $CURDIR
cp $DIR/evernym-react-native-sdk*.tgz $OUTPUTDIR
