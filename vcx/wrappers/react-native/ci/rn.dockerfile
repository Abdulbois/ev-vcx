# Development
FROM libindy-ubuntu18
ARG uid=1000
RUN useradd -ms /bin/bash -u $uid react-native

WORKDIR vcx/wrappers/react-native
# Assumes we are in the ./vcx directory
RUN npm i npm@6.1.0
COPY vcx/libvcx/target/debian/*.deb .
RUN dpkg -i *.deb
USER rn