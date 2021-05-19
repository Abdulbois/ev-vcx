export PATH=${PATH}:$(pwd)/vcx/ci/scripts

OUTPUTDIR=output
DIR=vcx/wrappers/react-native
CURDIR=$(pwd)

cd $DIR
npm i
npm run compile
npm pack

rename \s/rn-vcx-wrapper-/rn-vcx-wrapper_/ *.tgz
rename \s/\\.tgz\$/_amd64\\.tgz/ *.tgz

find . -type f -name 'rn-vcx-wrapper*.tgz' -exec create_npm_deb.py {} \;

cd $CURDIR
cp $DIR/rn-vcx*.tgz $OUTPUTDIR
cp $DIR/rn-vcx-wrapper_*.deb $OUTPUTDIR

