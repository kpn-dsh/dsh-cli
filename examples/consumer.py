from confluent_kafka import Consumer
import sys

PKI_CLIENT_KEY_LOCATION = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/vol/client.key"
PKI_CLIENT_CERTIFICATE_LOCATION = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/vol/client.pem"
PKI_CA_CERTIFICATE_LOCATION = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/vol/ca.pem"

CLIENT_ID = "greenbox-dev"
GROUP_ID = "greenbox-dev_vol_0"
BROKERS = "vol-0.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091,vol-1.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091,vol-2.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091"


def main():
    if len(sys.argv) < 2:
        print("missing topic argument", file=sys.stderr)
        exit(1)
    topic = sys.argv[1]

    kafka_config = {
        "auto.offset.reset": "earliest",
        "bootstrap.servers": BROKERS,
        "client.id": CLIENT_ID,
        "group.id": GROUP_ID,
        "security.protocol": "ssl",
        "ssl.ca.location": PKI_CA_CERTIFICATE_LOCATION,
        "ssl.certificate.location": PKI_CLIENT_CERTIFICATE_LOCATION,
        "ssl.key.location": PKI_CLIENT_KEY_LOCATION
    }

    consumer = Consumer(kafka_config)
    consumer.subscribe([topic])

    try:
        while True:
            topic_full = True
            while topic_full:
                msg = consumer.poll(1.0)
                if msg is not None:
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
