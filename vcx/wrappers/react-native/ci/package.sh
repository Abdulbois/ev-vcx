OUTPUTDIR=output
DIR=vcx/wrappers/react-native
CURDIR=$(pwd)

cd $DIR
npm i
npm run build
npm pack

rename \s/react-native-vcx-wrapper-/react-native-vcx-wrapper_/ *.tgz

cd $CURDIR
cp $DIR/react-native-vcx-wrapper*.tgz $OUTPUTDIR
