# Code example `Python consumer`

For this example we will create a `consumer` example for the `Python` programming language:

```shell
> dsh proxy code my-proxy python consumer
generating python consumer example for bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
created directory 'my-proxy-consumer-python-example'
created file 'my-proxy-consumer-python-example/my-proxy-consumer.py'
python code for bundle 'my-proxy' generated in directory 'my-proxy-consumer-python-example'
```

As is shown in the output, the example is generated in a newly created directory which contains
a Python script named `my-proxy-consumer.py`.

```python
from confluent_kafka import Consumer
import sys

PKI_DIRECTORY = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy"

CLIENT_ID = "my-tenant"
GROUP_ID = "my-tenant_my-proxy_1"
BROKERS = [
    "my-proxy-0.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-1.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-2.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
]


def main():
    if len(sys.argv) < 2:
        print("missing topic argument", file=sys.stderr)
        exit(1)
    topic = sys.argv[1]

    kafka_config = {
        "auto.offset.reset": "earliest",
        "bootstrap.servers": ",".join(BROKERS),
        "client.id": CLIENT_ID,
        "group.id": GROUP_ID,
        "security.protocol": "ssl",
        "ssl.ca.location": f"{PKI_DIRECTORY}/ca.pem",
        "ssl.certificate.location": f"{PKI_DIRECTORY}/client.pem",
        "ssl.key.location": f"{PKI_DIRECTORY}/client.key"
    }

    consumer = Consumer(kafka_config)
    consumer.subscribe([topic])

    try:
        while True:
            topic_full = True
            while topic_full:
                msg = consumer.poll(1.0)
                if msg is not None:
                    if msg.key() is not None:
                        print(f"{msg.partition()}:{msg.offset()} {msg.key().decode()}")
                else:
                    topic_full = False
    except KeyboardInterrupt:
        print("interrupted")
    finally:
        if consumer:
            consumer.close()


if __name__ == "__main__":
    main()
```

To run the example, we first change to the created directory, create a virtual environment and
install the `confluent_kafka` client library.

```shell
> cd my-proxy-consumer-python-example
> python3 -m venv .venv
> . .venv/bin/activate
(.venv) my-proxy-consumer-python-example> python3 -m pip install confluent_kafka
```

Now finally we can run the script and receive messages from a Kafka topic. In the example below
we use the same topic as for the [`python producer`](python-producer.md) example
(`scratch.example.my-tenant`). Use `ctrl-c` to stop the program.

```shell
(.venv) my-proxy-consumer-python-example> python3 my-proxy-consumer.py scratch.example.my-tenant
0:1999 timestamp: 1779997678
0:2000 timestamp: 1779997679
0:2001 timestamp: 1779997680
0:2002 timestamp: 1779997681
^C
```
