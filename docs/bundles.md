## Python example

```shell
> dsh bundle code vol --language python --output-directory ~/tmp
> cd ~/tmp/vol-python-example
> python3 -m venv .venv
> . .venv/bin/activate
(.venv) vol-python-example> python3 -m pip install confluent_kafka
(.venv) vol-python-example> ls -al
total 8
drwxr-xr-x 7 wilbert staff 224 8 mei 15:26 .venv/
-rw-r--r-- 1 wilbert staff 1693 8 mei 15:23 vol-consumer.py
(.venv) vol-python-example> python3 vol-consumer.py scratch.reference-implementation-avro.greenbox-dev
```

## Rust example

```shell
> dsh bundle code vol --language rust --output-directory ~/tmp
> cd ~/tmp/vol-rust-example
> cargo install --path .
> vol-consumer scratch.reference-implementation-avro.greenbox-dev
```
