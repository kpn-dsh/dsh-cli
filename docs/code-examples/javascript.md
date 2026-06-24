# `JavaScript` code example

> It is assumed that you have a recent version of `node.js` and its `npm` package manager
> installed.   
> The generated code has a dependency on
> [Confluent JavaScript Client for Apache Kafka](https://docs.confluent.io/kafka-clients/javascript/current/overview.html).

We will create the example code for the `JavaScript` programming language with the following
command:

```shell
> dsh proxy code my-proxy javascript
generating javascript example for bundle 'my-proxy' for 'np-aws-lz-dsh@greenbox-dev'
created directory 'my-proxy-example-javascript'
created file 'my-proxy-example-javascript/package.json'
created file 'my-proxy-example-javascript/consumer.js'
created file 'my-proxy-example-javascript/list-topics.js'
created file 'my-proxy-example-javascript/producer.js'
javascript code for bundle 'my-proxy' generated in directory 'my-proxy-example-javascript'
```

As is shown in the output, the example is generated in a newly created directory which contains
three `JavaScript` scripts named `consumer.js`, `list-topics.js` and `producer.js` and an `npm`
configuration file named `package.json` (which contains information about the dependencies).
Click below to see the generated code.

<details>
<summary><code>package.json</code></summary>

```json
{
  "dependencies": {
    "@confluentinc/kafka-javascript": "1.9.x",
    "process": "0.11.x"
  }
}
```

</details>

<details>
<summary><code>consumer.js</code></summary>

```javascript
const {Kafka, logLevel} = require('@confluentinc/kafka-javascript').KafkaJS;
const process = require('process');

const pkiDirectory = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/my-proxy";
const clientId = "greenbox-dev";
const groupId = "greenbox-dev_my-proxy_1";
const brokers = [
    "my-proxy-0.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-1.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-2.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091"
];

async function main() {
    // Allow handling of ctrl-c
    process.on('SIGTERM', () => handle_termination('SIGTERM'));
    process.on('SIGINT', () => handle_termination('SIGINT'));

    let topic = process.argv[2];
    if (!topic) {
        console.error("missing topic argument");
    }

    const kafkaConfig = {
        kafkaJS: {
            clientId,
            brokers,
            ssl: true,
            logLevel: logLevel.ERROR
        },
        "ssl.ca.location": pkiDirectory + "/ca.pem",
        "ssl.certificate.location": pkiDirectory + "/client.pem",
        "ssl.key.location": pkiDirectory + "/client.key"
    };

    const kafka = new Kafka(kafkaConfig);
    let consumer = kafka.consumer({
        kafkaJS: {
            groupId,
            sessionTimeout: 30000,
            heartbeatInterval: 3000,
        }
    });
    await consumer.connect();
    await consumer.subscribe({topic: topic});

    await consumer.run({
        eachMessage: async ({_, partition, message}) => {
            console.log(partition + ':' + message.offset + ' ' + message.key.toString());
        },
    });
}

function handle_termination(signal) {
    console.log(signal);
    process.exit();
}

if (require.main === module) {
    main().catch(console.error);
}
```

</details>


<details>
<summary><code>list-topics.js</code></summary>

```javascript
const {Kafka, logLevel} = require('@confluentinc/kafka-javascript').KafkaJS;
const process = require('process');

const pkiDirectory = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/my-proxy";
const clientId = "greenbox-dev";
const groupId = "greenbox-dev_my-proxy_1";
const brokers = [
    "my-proxy-0.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-1.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-2.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091"
];

async function main() {
    const kafkaConfig = {
        kafkaJS: {
            clientId,
            brokers,
            ssl: true,
            logLevel: logLevel.ERROR
        },
        "ssl.ca.location": pkiDirectory + "/ca.pem",
        "ssl.certificate.location": pkiDirectory + "/client.pem",
        "ssl.key.location": pkiDirectory + "/client.key"
    };

    const kafka = new Kafka(kafkaConfig);
    let consumer = kafka.consumer({
        kafkaJS: {
            groupId,
            sessionTimeout: 30000,
            heartbeatInterval: 3000,
        }
    });
    await consumer.connect();
    let admin = consumer.dependentAdmin();
    await admin.connect();
    let topics = await admin.listTopics();
    topics.sort();
    topics.forEach(topic => console.log(topic));
    process.exit();
}

if (require.main === module) {
    main().catch(console.error);
}
```

</details>

<details>
<summary><code>producer.ts</code></summary>

```javascript
const {Kafka, logLevel} = require('@confluentinc/kafka-javascript').KafkaJS;
const process = require('process');

const pkiDirectory = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/my-proxy";
const clientId = "greenbox-dev";
const brokers = [
    "my-proxy-0.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-1.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-2.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091"
];

let producerInterval = 0;

async function main() {
    // Allow handling of ctrl-c
    process.on('SIGTERM', () => handle_termination('SIGTERM'));
    process.on('SIGINT', () => handle_termination('SIGINT'));

    let topic = process.argv[2];
    if (!topic) {
        console.error("missing topic argument");
    }

    const kafkaConfig = {
        kafkaJS: {
            clientId,
            brokers,
            ssl: true,
            logLevel: logLevel.ERROR
        },
        "ssl.ca.location": pkiDirectory + "/ca.pem",
        "ssl.certificate.location": pkiDirectory + "/client.pem",
        "ssl.key.location": pkiDirectory + "/client.key"
    };

    const kafka = new Kafka(kafkaConfig);
    let producer = kafka.producer({
        kafkaJS: {}
    });
    await producer.connect();

    producerInterval = setInterval(async () => {
        const key = "my-proxy-producer-javascript: " + Math.round(new Date().getTime() / 1000);
        await producer.send({
            topic,
            messages: [
                {
                    key,
                    value: "payload",
                },
            ],
        });
        console.log(key);
    }, 1000)
}

function handle_termination(signal) {
    console.log(signal);
    if (producerInterval) {
        clearInterval(producerInterval);
    }
    process.exit();
}

if (require.main === module) {
    main().catch(console.error);
}
```

</details>

To run the example scripts, we first change to the created directory and install the
`@confluentinc/kafka-javascript` client library (if it is not already installed):

```shell
> cd my-proxy-example-javascript
> npm install "@confluentinc/kafka-javascript"
...
```

Now finally we are able to run the scripts. First we will list all Kafka topics that we have
read access to.

```shell
> node list-topics
scratch.my-topic.my-tenant
...
```

Now we can send some messages to a Kafka topic. In the example below we use topic
`scratch.my-topic.my-tenant`. Use `ctrl-c` to stop the program.

```shell
> node producer scratch.my-topic.my-tenant
my-proxy-producer-javascript: 1782301678
my-proxy-producer-javascript: 1782301679
my-proxy-producer-javascript: 1782301680
my-proxy-producer-javascript: 1782301681
my-proxy-producer-javascript: 1782301682
^CSIGINT
```

Next we will consume records from the topic. Again, use `ctrl-c` to stop the program.

```shell
> node consumer scratch.example.greenbox-dev
0:3099 my-proxy-producer-javascript: 1782301678
0:3100 my-proxy-producer-javascript: 1782301679
0:3101 my-proxy-producer-javascript: 1782301680
0:3102 my-proxy-producer-javascript: 1782301681
0:3103 my-proxy-producer-javascript: 1782301682
^CSIGINT
```

[&#x2190; Kafka proxy](../proxy.md)
