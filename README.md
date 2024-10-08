# Pika 
This is a simple console app that allows syncing files between my Mac and Windows computers asynchronously, so both do not have to be running for it to work. This is backed by a raspberrypi running the localstack docker container, which is always running at my house.

# Usage 

```
pika up file.txt
pika up dir # will zip and upload
pika down # will download all files with matching prefix
```

# Usage
`cargo build --release && ./target/release/pika`
