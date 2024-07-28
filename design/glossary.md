# Glossary

## Resource

Exists independent of Trifonius.

### Static resource characteristics

<table>
  <tr>
    <td>persistent</td>
    <td>
    </td>
</tr>
  <tr>
    <td>realtime</td>
    <td>
    </td>
</tr>
  <tr>
    <td>source/sink</td>
    <td>
    </td>
</tr>
  <tr>
    <td>producer/consumer</td>
    <td>
    </td>
</tr>
  <tr>
    <td>push/pull</td>
    <td>
    </td>
</tr>
  <tr>
    <td>trigger</td>
    <td>
    </td>
</tr>
  <tr>
    <td>request/response</td>
    <td>
    </td>
</tr>
</table>

### Dynamic resource characteristics

<table>
  <tr>
    <td>realtime</td>
    <td>
    </td>
</tr>
  <tr>
    <td>source/sink</td>
    <td>
    </td>
</tr>
  <tr>
    <td>producer/consumer</td>
    <td>
    </td>
</tr>
  <tr>
    <td>push/pull</td>
    <td>
    </td>
</tr>
  <tr>
    <td>trigger</td>
    <td>
    </td>
</tr>
  <tr>
    <td>request/response</td>
    <td>
    </td>
</tr>
</table>

| dynamic           |   |
|-------------------|---|
| producer/consumer |   |
| push/pull         |   |
| realtime/batch    |   |
| request/response  |   |
| source/sink       |   |
| trigger           |   |

### Resource types

#### Kafka topic

* Realtime
* Can be source and sink

#### Kafka MQTT/HTTP gateway

Similar to Kafka topic

* Realtime
* Can be source and sink

#### Bucket resource

* Batch
* Can be source and sink

#### Database resource

* Batch
* Can be source and sink

#### REST Api

* Batch
* Can be source and sink

## Processor

* Inbound junctions
* Outbound junctions
* Deployment settings

### Processor types

#### DSH Service

#### DSH App

#### Trifonius processor

## Topology

A _Topolgy_ defines how _Resources_ and _Processors_ are connected together.

### Topology Layout

Although the _Topology Layout_ strictly is not not logically

### Topology Persistence

A _Topolgy_ _Layout_

### Topology types

#### Pipeline

A _Pipeline_ defines a topology

resources

* writable kafka topic
  source
* readable kafka topic
  processor
* consent filter
  pipeline
* components
* connections between components
* positioning on screen

pipeline positioning data separate

pipeline

* how components are connected
* how components are positioned (stored separate from the connections)
  component
  tenant
  target tenant
  secret
  resource
* source resource
* sink resource
* kafka topic resource, can be source resource or sink resource
* s3/bucket resource, can be source resource or sink resource
* database resource, can be source resource or sink resource
  dsh service
  processor
  topic resource

GET /resources all resources plus description (only kafka topics for now)
GET /resources/source
GET /resources/sink
GET /processors all processors plus description (only consentfilter for now)
POST /pipeline/[pipeline-id]/create  
PUT /pipeline/[pipeline-id]/deploy body contains pipeline definition
DELETE /pipeline/[pipeline-id]/stop
GET /pipeline/[pipeline-id]/status
