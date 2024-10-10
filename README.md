# Pika 
This is a simple console application meant to allow syncing of certain directories between my windows and Mac. There are obviously better programs out there for this, but I wanted to build something myself that I can iterate on later.

The app runs as a daemon on both computers and listens for changes to the source path (defined in config.json). Once a new file arrives there, it is written to S3 which is running on a raspberry pi. Periodically, the other computer will read S3 and download any new files to the destination path (also defined in config.json).

# Usage
`cargo build --release && ./target/release/pika`

# Setting up as daemon

