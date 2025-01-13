# Pika 
This is a simple console app that allows syncing files between my Mac and Windows computers asynchronously, so both do not have to be running for it to work. This is backed by a raspberrypi running the localstack docker container, which is always running at my house.

# Usage 

```
pika -u file.txt
pika -u dir # will zip and upload
pika -d # will download all files with matching prefix
```

# Updating on Mac or WSL
```
cargo build --debug # --release
cp target/debug/pika /usr/local/bin
cp config.json ~/.pika/config.json
```
