# Code example `Python`

For this example we will create example code for the `Python` programming language:

```shell
> dsh proxy code my-proxy python
generating python example for bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
created directory 'my-proxy-example-python'
created file 'my-proxy-example-python/my-proxy-consumer.py'
created file 'my-proxy-example-python/my-proxy-producer.py'
python code for bundle 'my-proxy' generated in directory 'my-proxy-example-python'
```

As is shown in the output, the example is generated in a newly created directory which contains
two Python scripts named `my-proxy-consumer.py` and `my-proxy-producer.py`. Click below to see
the generated code.

<details>
<summary><code>my-proxy-consumer.py</code></summary>

```python
from confluent_kafka import Consumer
import sys

PKI_DIRECTORY = "/Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy"

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

</details>

<details>
<summary><code>my-proxy-producer.py</code></summary>

```python
from confluent_kafka import Producer
from datetime import datetime
import math
import sys
import time

PKI_DIRECTORY = "/Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy"
CLIENT_ID = "my-tenant"
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
        "bootstrap.servers": ",".join(BROKERS),
        "client.id": CLIENT_ID,
        "security.protocol": "ssl",
        "ssl.ca.location": f"{PKI_DIRECTORY}/ca.pem",
        "ssl.certificate.location": f"{PKI_DIRECTORY}/client.pem",
        "ssl.key.location": f"{PKI_DIRECTORY}/client.key"
    }

    producer = Producer(kafka_config)

    try:
        while True:
            key = f"my-proxy-producer-python: {math.floor(datetime.now().timestamp())}"
            producer.produce(topic=topic, key=key)
            producer.flush()
            print(key)
            time.sleep(1)
    except KeyboardInterrupt:
        print("interrupted")
    finally:
        producer.flush()


if __name__ == "__main__":
    main()
```

</details>


To run the example scripts, we first change to the created directory, create a virtual environment
and install the `confluent_kafka` client library.

```shell
> cd my-proxy-example-python
> python -m venv .venv
> . .venv/bin/activate
(.venv) my-proxy-example-python> python -m pip install confluent_kafka
```

Now finally we can run the scripts and send and receive messages to and from a Kafka topic.
In the example below we use topic `scratch.example.my-tenant`. First we will produce some records.
Use `ctrl-c` to stop the program.

```shell
(.venv) my-proxy-example-python> python3 my-proxy-producer.py scratch.example.greenbox-dev
my-proxy-producer-python: 1780045672
my-proxy-producer-python: 1780045674
my-proxy-producer-python: 1780045675
my-proxy-producer-python: 1780045676
my-proxy-producer-python: 1780045677
^Cinterrupted
```

Next we will consume records from the topic. Again, use `ctrl-c` to stop the program.

```shell
(.venv) my-proxy-example-python> python3 my-proxy-consumer.py scratch.example.greenbox-dev
0:2729 my-proxy-producer-python: 1780045672
0:2730 my-proxy-producer-python: 1780045674
0:2731 my-proxy-producer-python: 1780045675
0:2732 my-proxy-producer-python: 1780045676
0:2733 my-proxy-producer-python: 1780045677
^Cinterrupted
```