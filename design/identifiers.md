# Technologies, realizations and identifiers

## Glossary

<table>
    <tr style="vertical-align: top;">
        <th>term</th>
        <th>description</th>
        <th>examples</th>
        <th>unique id</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Processor</em></td>
        <td>
            A Trifonius <em>Pipeline</em> is a set of collaborating <em>Processor</em>s and/or 
            <em>Resource</em>s, together with a specification how these components are 
            connected together and how they are configured.
            A deployed and started <em>Pipeline</em> can realize a requested capability.
        </td>
        <td><code>itv-pipeline</code></td>
        <td><code>PipelineId</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Processor Technology</em></td>
        <td>
            A <em>Processor Technology</em> is a technical solution
            for the realization of a Trifonius <em>Processor</em>.
            Examples are DSH services or apps from the DSH App Catalog.
            Support for more technology solutions will be added later,
            e.g. Flink, Polars, Nifi, et cetera.
        </td>
        <td><code>dsh-app</code><br/><code>dsh-service</code></td>
        <td><code>ProcessorTechnology</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Processor Realization</em></td>
        <td>
            A <em>Processor Realization</em> is a <em>Processor</em> 
            that is available for use by Trifonius, 
            and will show up in the <em>Processor</em> catalog or registry.
            A <em>Processor Realization</em> is always implemented using a technical solution 
            based on one of the <em>Processor Technology</em>s, 
            e.g. a container in the DSH Registry. 
            A <em>Processor Realization</em> is usually realized by building and deploying 
            a technical component based on the <em>Processor Technology</em> and 
            by specifying its behavior, characteristics and deployment requirements
            in a configuration file.
        </td>
        <td><code>replicator</code></td>
        <td><code>ProcessorRealizationId</code><br/>+ <code>Version</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Processor Instance</em></td>
        <td>
            A <em>Processor Instance</em> is a configured <em>Processor Realization</em> 
            used in a <em>Pipeline</em>.
            In order to use the <em>Processor Realization</em>, 
            the <em>Pipeline</em> must assign the instance a <code>ProcessorInstanceId</code>,
            which must be unique in the scope of the <em>Pipeline</em>. 
            The <em>Pipeline</em> must also specify all the deployment requirements 
            of the <em>Processor Realization</em>, and provide a human friendly name.
        </td>
        <td><code>itv-replicator</code></td>
        <td><code>ProcessorInstanceId</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Resource Technology</em></td>
        <td>
            A <em>Resource Technology</em> is the technical solution
            which implements a source/sink resource that is available to the Trifonius 
            <em>Processor</em>s. 
            At this time the only available <em>Resource Technology</em> 
            are the Kafka topics managed by the DSH.
            Support for more technology solutions will be added later,
            e.g. S3 buckets, databases, web-services, et cetera.
        </td>
        <td><code>dsh-topic</code></td>
        <td><code>ResourceTechnology</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Resource Realization</em></td>
        <td>
            A <em>Resource Realization</em> is a source/sink resource 
            that is available for use by Trifonius, 
            and will show up in the <em>Resource</em> catalog or registry.
            A <em>Resource Realization</em> is always implemented using a technical solution 
            based on one of the <em>Resource Technology</em>s, 
            e.g. a Kafka topic managed by DSH.
            The set of available <em>Processor Realization</em>s typically 
            originates outside Trifonius, and is defined by the current installed base 
            of the <em>Technical Resource</em>. 
            E.g., the set of <em>Resource</em>s for the <em>Technology Resource</em> 
            <code>dsh-topic</code> consists of the set of Kafka topics accessible to the tenant.
        </td>
        <td><code>stb-status</code></td>
        <td><code>ResourceRealizationId</code><br/> + <code>Version</code> (optional)</td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Resource Instance</em></td>
        <td>
            A <em>Resource Instance</em> is a configured <em>Resource Realization</em> 
            used in a <em>Pipeline</em>.
            In order to use the <em>Resource Realization</em>, 
            the <em>Pipeline</em> must assign the instance a <code>ResourceInstanceId</code>,
            which must be unique in the scope of the <em>Pipeline</em>.
            The <em>Pipeline</em> must also specify all the deployment requirements 
            of the <em>Resource Realization</em> (if any), and provide a human friendly name.
        </td>
        <td><code>status-topic</code></td>
        <td><code>ResourceInstanceId</code></td>
    </tr>
</table>

* Pipelines
    * `PipelineId` - Uniquely identifies a pipeline.
    * `PipelineName` - Human friendly name of a pipeline.
* Processors
    * `ProcessorTechnology` - Identifies the technical solutions that realize processors.
    * `ProcessorRealization` - Identifies a specific realization of a processor,
      based on a specific processor technology.
    * `JunctionId` - Identifies the junctions in a processor realization,
      that connects a processor to resources or other processors.
    * `ParameterId` - Identifies the parameters that the pipeline designer must provide
      when using a processor in a pipeline.
    * `ProcessorId` - Identifies the definition/use of a processor realization
      in the scope of a pipeline.
    * `ProcessorName` - Human friendly name of the use of a processor in the definition of a
      pipeline.
* Resources
    * `ResourceTechnology` - Identifies the technical solutions that realize resources.
    * `ResourceRealization` - Identifies a specific realization of a resource,
      based on a specific resource technology.
    * `ResourceId` - Identifies the definition/use of a resource realization
      in the scope of a pipeline.
    * `ResourceName` - Human friendly name of the use of a resource in the definition of a
      pipeline.

## Pipelines

### `PipelineId`

A `PipelineId` uniquely identifies a defined and saved Trifonius _Pipeline_.
The `PipelineId` is generated by the backend/engine when a _Pipeline_ is created for the first time.
A `PipelineId` is a meaningless string with some syntactical restrictions,
and cannot be changed once it is generated.

### `PipelineName`

The `PipelineName` is the human friendly name for a _Pipeline_,
which will be used in the interaction with the _Pipeline_ designer.
The `PipelineName` is defined by the _Pipeline_ designer and
it can contain all utf-8 characters (although smileys et cetera should be avoided).

## Processors

### `ProcessorTechnology`

A `ProcessorTechnology` defines one technical solution
for the realization of a Trifonius _Processor_.
Examples are DSH services or DSH Apps (from the App Catalog).
Other technology solution can be added later.

`ProcessorTechnology` types are defined by the Trifonius framework,
so there is a more or less static set of recognized values.
Adding a new `ProcessorTechnology` type requires designing,
developing and merging new code to the Trifonius engine.
It typically will not require any changes to the backend or the frontend,
unless the new `ProcessorTechnology` requires capabilities that are not yet supported
by the current generic implementations of the backend or frontend.

<table>
    <tr style="vertical-align: top;">
        <th><code>ProcessorTechnology</code></th>
        <th>description</th>
    </tr>
    <tr style="vertical-align: top;  color: gray;">
        <td><code>dsh-app</code></td>
        <td>
          DSH App <em>Processor</em>s make Apps published to the DSH App Catalog eligible
          for deployment and control via Trifonius. This type is not yet supported, 
          but is planned for the near future.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>dsh-service</code></td>
        <td>
          DSH Service <em>Processor</em>s make containers published in the DSH container registry 
          (Harbor) eligible for deployment and control via Trifonius.
        </td>
    </tr>
</table>

### `ProcessorRealization`

### `JunctionId`

### `ParameterId`

### `ProcessorId`

### `ProcessorName`

## Resources

### `ResourceTechnology`

A `ResourceTechnology` defines one technical solution
for the realization of a Trifonius _Sink_ or _Source_ _Processor_.
An example is a DSH topic.

`ResourceTechnology` types are defined by the Trifonius framework,
so there is a more or less static set of recognized values.
Adding a new `ResourceTechnology` type requires designing,
developing and merging new code to the Trifonius engine.
It typically will not require any changes to the backend or the frontend,
unless the new `ResourceTechnology` type requires capabilities that are not yet supported
by the current generic implementations of the backend or frontend.

<table>
    <tr style="vertical-align: top;">
        <th><code>ResourceTechnology</code></th>
        <th>description</th>
    </tr>
    <tr style="vertical-align: top;  color: gray;">
        <td><code>dsh-gateway</code></td>
        <td>
          DSH gateway <em>Resource</em>s make DSH stream topics, connected to the DSH gateway,
          eligible for use and control via Trifonius. This type is not yet supported, 
          but is planned for the near future.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>dsh-topic</code></td>
        <td>
          DSH topic <em>Resource</em>s make ordinary DSH Kafka topics eligible for use and 
          control via Trifonius.
        </td>
    </tr>
</table>

## `Id`s

`Id`s related to _Processors_ are typically defined in the configuration files
for these _Processors_ (e.g. to identify a junction or a parameter).
The configuration will be read by the Trifonius engine, either at startup time or dynamic.
Adding a new _Processor_ requires writing a new configuration file,
that makes a DSH service or app eligible for deployment and control via Trifonius.
Building these DSH services and apps, and pushing/publishing them to the
container registry or app catalog is independent of Trifonius.
Defining new _Processor_ `Id`s will typically not require any changes to the engine,
backend or frontend.

`Id`s related to _Resources_ are more tightly bound to the Trifonius framework than `Id`s
related to _Processors_. Adding a new _Resource_ type often also means that that
new _Resource_ requires new `Id`s. Again, defining new _Resource_ `Id`s will typically not
require any changes to the backend or frontend.

All `Id`s are strictly validated, so for each type of `Id` a regular expression is given.

<table>
    <tr style="vertical-align: top;">
        <th>identifier</th>
        <th>description</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>JunctionId</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9-]{0,29}$</code><br/>
            <code>JunctionId</code>s identify inbound or outbound <em>Junction</em>s in the scope 
            of a <em>Processor</em>. 
            <code>JunctionId</code>s must be unique within the scope of their containing 
            <em>Processor</em>, so an inbound <em>Junction</em> can not have the same 
            <code>JunctionId</code> as an outbound <em>Junction</em>.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ParameterId</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9-]{0,29}$</code><br/>
            A <code>ParameterId</code> identifies a <em>Parameter</em> that must be provided
            by the pipeline designer when he is designing a pipeline.
            <em>Parameter</em>s (and hence the <code>ParameterId</code>s) are defined in the 
            <em>Processor</em> configuration and can be retrieved from the backend as part 
            of the <em>Processor</em> descriptor.
            <code>ParameterId</code>s must be unique within the scope of their 
            containing <em>Processor</em>.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorId</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9-]{0,29}$</code><br/>
            A <code>ProcessorId</code> identifies a <code>ProcessorRealization</code>, 
            which is defined by a <code>Processor</code> together with its configuration. 
            (Thus, a <code>ProcessorId</code> does not identify a <em>ProcessorInstance</em>.)
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProfileId</code></td>
        <td>
            <em>regex</em>: <code>^[a-z0-9-]{0,29}$</code><br/>
            A <code>ProfileId</code> identifies a <em>Profile</em> declared within the scope a 
            <em>Processor</em>. <code>ProfileId</code>s must be unique within the scope of their 
            containing <em>Processor</em>.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceId</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9-]{0,29}$</code><br/>
            A <code>ResourceId</code> identifies a <code>ResourceRealization</code>, 
            which is defined by a <code>Resource</code> together with its configuration. 
            (Thus, a <code>ResourceId</code> does not identify a <em>ResourceInstance</em>.)
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>TaskId</code></td>
        <td>
            <em>regex</em>: <code>^.*$</code><br/>
        </td>
    </tr>
</table>

## Identifiers

While the `Id`s describes above identify their components within the scope of their
containing components, `Identifier`s are used to identify _Processors_, _Junctions_ and
_Resources_ in the scope of their respective registries.
E.g., a `ProcessorIdentifier` like `(dsh-service, greenbox-consent-filter)` can be used to
request the _Processor_ registry for a `dsh-service` _Processor_ configured for the
`greenbox-consent-filter` realization.

<table>
    <tr style="vertical-align: top;">
        <th>identifier</th>
        <th>description</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>JunctionIdentifier</code></td>
        <td>
            <em>tuple</em>: <code>(ProcessorIdentifier, JunctionId)</code><br/>
            <em>tuple</em>: <code>(ProcessorType, ProcessorId, JunctionId)</code><br/>
            A <code>JunctionId</code> identifies an inbound or outbound junction in a 
            <em>Processor</em>. <code>JunctionId</code>s must be unique within the scope of 
            their containing <em>Processor</em>, 
            so an inbound junction can not have the same id 
            as an outbound junction.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorIdentifier</code></td>
        <td>
            <em>tuple</em>: <code>(ProcessorType, ProcessorId)</code><br/>
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceIdentifier</code></td>
        <td>
            <em>tuple</em>: <code>(ResourceType, ResourceId)</code><br/>
            A <code>JunctionId</code> identifies an inbound or outbound junction in a 
            <em>Processor</em>. <code>JunctionId</code>s must be unique within the scope of 
            their containing <em>Processor</em>, 
            so an inbound junction can not have the same id 
            as an outbound junction.
        </td>
    </tr>
</table>

## Names

Names, sometimes called "given names", identify instances of _Pipelines_, _Processors_ or
_Resources_.

Given names are defined by the pipeline designer via the Trifonius frontend, when designing a
pipeline.
Since the `PipelineName` and the `ProcessorName` are used together to construct the name
of the deployed DSH service, these names must adhere to the strict naming constraints
of the DSH platform. Most notable, this means that

* they cannot contain any special characters, like hyphen (`-`) or underscore (`_`),
* they must be all lowercase and
* their length is restricted to 18 characters.

<table>
    <tr style="vertical-align: top;">
        <th>identifier</th>
        <th>description</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>DshServiceName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}(-[a-z][a-z0-9]{0,17})?$</code><br/>
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}$</code><br/>
            A <code>PipelineName</code> identifies a declared and saved <em>Pipeline</em>
            as it will be managed by Trifonius.
            Since the <code>PipelineName</code> is the primary key of a <em>Pipeline</em> in the 
            <em>Pipeline</em> registry, it cannot be changed once it is defined. 
            However, for the representation of a <em>Pipeline</em> to the pipeline designer 
            via the frontend, 
            the <em>Pipeline</em> label field can be used for a more user-friendly name. 
            This label can be changed after it was defined.
            The <code>PipelineName</code> will be generated by the backend when a 
            <em>Pipeline</em> is first 
            declared/saved. The backend will attempt to generate a more or less readable name,
            based on the <em>Pipeline</em> label that was provided with the first declaration.
            The <code>PipelineName</code> will also be used to store the layout 
            of the <em>Pipeline</em> in de layout-backend.
            The <code>PipelineName</code> will, together with the 
            <code>ProcessorName</code>, be used to generate 
            the name of a deployed service on the DSH (<code>PipelineProcessorName</code>).
            Therefor it needs to adhere to strict rules regarding length and syntax.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}$</code><br/>
            The <code>ProcessorName</code> identifies a <em>Processor</em> within the scope 
            of a <em>Pipeline</em>, as it will be managed by Trifonius.
            Since the <code>ProcessorName</code> (together with the <code>PipelineName</code>) 
            is the primary key of a <em>Processor</em> in the pipeline registry, 
            it cannot be changed once it is defined. 
            However, for the representation of a <em>Processor</em> to the pipeline designer 
            via the frontend, the <em>Processor</em> label field can be used for a more 
            user-friendly name. This label can be changed after it was defined.
            A <code>ProcessorName</code> identifies a declared and saved <em>Processor</em> 
            component in the scope of a declared and saved pipeline.
            A <code>ProcessorName</code> can be defined by the Trifonius pipeline designer. 
            when a new <em>Pipeline</em> is received from the client, 
            The <code>ProcessorName</code> will also be used as a subkey to store the position 
            of the <em>Processor</em> in de layout-backend.
            <br/>
            The <code>ProcessorName</code> will, together with the 
            <code>PipelineName</code>, be used to generate 
            the name of a deployed service on the DSH (<code>PipelineProcessorName</code>).
            Therefor it needs to adhere to strict rules regarding length and syntax.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineProcessorName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}-[a-z][a-z0-9]{0,17}$</code><br/>
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineResourceName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}-[a-z][a-z0-9]{0,17}$</code><br/>
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}$</code><br/>
            The <code>ResourceName</code> identifies a <em>Resource</em> within the scope 
            of a <em>Pipeline</em>, as it will be managed by Trifonius.
            Since the <code>ResourceName</code> (together with the <code>PipelineName</code>) 
            is the primary key of a <em>Resource</em> in the pipeline registry, 
            it cannot be changed once it is defined. 
            However, for the representation of a <em>Resource</em> to the pipeline designer 
            via the frontend, the <em>Resource</em> label field can be used for a more 
            user-friendly name. This label can be changed after it was defined.
            A <code>ResourceName</code> identifies a declared and saved <em>Resource</em> 
            component in the scope of a declared and saved pipeline.
            A <code>ResourceName</code> can be defined by the Trifonius pipeline designer. 
            when a new <em>Pipeline</em> is received from the client, 
            The <code>ResourceName</code> will also be used as a subkey to store the position 
            of the <em>Resource</em> in de layout-backend.
            <br/>
        </td>
    </tr>
</table>

## Syntax

<table>
    <tr style="vertical-align: top;">
        <th>identifier</th>
        <th>length</th>
        <th>regex</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>JunctionId</code></td>
        <td><code>1 - 30</code></td>
        <td><code>^[a-z][a-z0-9-]{0,29}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ParameterId</code></td>
        <td><code>1 - 30</code></td>
        <td><code>^[a-z][a-z0-9-]{0,29}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorId</code></td>
        <td><code>1 - 30</code></td>
        <td><code>^[a-z][a-z0-9-]{0,29}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProfileId</code></td>
        <td><code>1 - 30</code></td>
        <td><code>^[a-z0-9-]{0,29}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceId</code></td>
        <td><code>1 - 30</code></td>
        <td><code>^[a-z][a-z0-9-]{0,29}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>TaskId</code></td>
        <td><code></code></td>
        <td><code>^.*$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>JunctionIdentifier</code></td>
        <td><code></code></td>
        <td>
            <em>tuple</em>: <code>(ProcessorIdentifier, JunctionId)</code><br/>
            <em>tuple</em>: <code>(ProcessorType, ProcessorId, JunctionId)</code>
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorIdentifier</code></td>
        <td><code></code></td>
        <td><em>tuple</em>: <code>(ProcessorType, ProcessorId)</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceIdentifier</code></td>
        <td><code></code></td>
        <td><em>tuple</em>: <code>(ResourceType, ResourceId)</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>DshServiceName</code></td>
        <td><code>1 - 37</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}(-[a-z][a-z0-9]{0,17})?$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineName</code></td>
        <td><code>1 - 18</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorName</code></td>
        <td><code>1 - 18</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineProcessorName</code></td>
        <td><code>1 - 37</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}-[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineResourceName</code></td>
        <td><code>1 - 37</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}-[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceName</code></td>
        <td><code>1 - 18</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
</table>
