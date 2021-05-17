#!/bin/bash
cd vcx/wrappers/react-native/
npm i
npm run lint
npm run compile
npm test
npm run test-logging