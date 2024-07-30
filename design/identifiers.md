# Types and identifiers

Trifonius uses many different types and identifier.

* _Processor_ types
* _Resource_ types
* `Id`s
* `Identifier`s
* `Name`s

## _Processor_ types

_Processor_ types (`ProcessorType`) are defined by the Trifonius framework,
so there is a more or less static set of recognized values.
Adding a new _Processor_ type requires designing,
developing and merging new code to the Trifonius engine.
It typically will not require any changes to the backend or the frontend,
unless the new _Processor_ requires capabilities that are not yet supported
by the current generic implementations of the backend or frontend.

<table>
    <tr style="vertical-align: top;">
        <th>processor type</th>
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

## _Resource_ types

_Resource_ types (`ResourceType`) are defined by the Trifonius framework,
so there is a more or less static set of recognized values.
Adding a new _Resource_ type requires designing,
developing and merging new code to the Trifonius engine.
It typically will not require any changes to the backend or the frontend,
unless the new _Resource_ type requires capabilities that are not yet supported
by the current generic implementations of the backend or frontend.

<table>
    <tr style="vertical-align: top;">
        <th>resource type</th>
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
            A <code>ProfileId</code> identifies a <em>Profile</em> declared within the scope a <em>Processor</em>. 
            <code>ProfileId</code>s must be unique within the scope of their containing <em>Processor</em>.
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
            A <code>JunctionId</code> identifies an inbound or outbound junction in a <em>Processor</em>. 
            <code>JunctionId</code>s must be unique within the scope of their containing <em>Processor</em>, 
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
            A <code>JunctionId</code> identifies an inbound or outbound junction in a <em>Processor</em>. 
            <code>JunctionId</code>s must be unique within the scope of their containing <em>Processor</em>, 
            so an inbound junction can not have the same id 
            as an outbound junction.
        </td>
    </tr>
</table>

## Names

Names, sometimes called "given names", identify instances of pipelines or <em>Processor</em>s.

Given names are defined by the pipeline designer via the Trifonius frontend, when designing a
pipeline.
Since the `PipelineName` and the `ProcessorName` are used together to construct the name
of the deployed DSH service, these names must adhere to the strict naming constraints
of the DSH platform. Most notable, this means that

* they cannot contain any special characters, like `-` or `_`,
* they must be all lowercase and
* their length is restricted to 19 characters.

<table>
    <tr style="vertical-align: top;">
        <th>identifier</th>
        <th>description</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}$</code><br/>
            A <code>PipelineName</code> identifies a declared and saved <em>Pipeline</em>
            as it will be managed by Trifonius.
            Since the <code>PipelineName</code> is the primary key of a <em>Pipeline</em> in the 
            <em>Pipeline</em> registry, it cannot be changed once it is defined. 
            However, for the representation of a <em>Pipeline</em> to the pipeline designer via the frontend, 
            the <em>Pipeline</em> label field can be used for a more user-friendly name. 
            This label can be changed after it was defined.
            The <code>PipelineName</code> will be generated by the backend when a <em>Pipeline</em> is first 
            declared/saved. The backend will attempt to generate a more or less readable name,
            based on the <em>Pipeline</em> label that was provided with the first declaration.
            The <code>PipelineName</code> will also be used to store the layout 
            of the <em>Pipeline</em> in de layout-backend.
            The <code>PipelineName</code> will, together with the 
            <code>ProcessorName</code>, be used to generate 
            the name of a deployed service on the DSH (<code>DshServiceName</code>).
            Therefor it needs to adhere to strict rules regarding length and syntax.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}$</code><br/>
            The <code>ProcessorName</code> identifies a <em>Processor</em> within the scope of a pipeline, 
            as it will be managed by Trifonius.
            Since the <code>ProcessorName</code> (together with the <code>PipelineId</code>) 
            is the primary key of a <em>Processor</em> in the pipeline registry, 
            it cannot be changed once it is defined. 
            However, for the representation of a <em>Processor</em> to the pipeline designer via the frontend, 
            the <em>Processor</em> label field can be used for a more user-friendly name. 
            This label can be changed after it was defined.
            A <code>ProcessorName</code> identifies a declared and saved <em>Processor</em> 
            component in the scope of a declared and saved pipeline.
            A <code>ProcessorName</code> can be defined by the Trifonius pipeline designer. 
            when a new <em>Pipeline</em> is received from the client, 
            The <code>ProcessorName</code> will also be used as a subkey to store the position 
            of the <em>Processor</em> in de layout-backend.
            <br/>
            The <code>ProcessorName</code> will, together with the 
            <code>PipelineName</code>, be used to generate 
            the name of a deployed service on the DSH (<code>DshServiceName</code>).
            Therefor it needs to adhere to strict rules regarding length and syntax.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ServiceName</code></td>
        <td>
            <em>regex</em>: <code>^[a-z][a-z0-9]{0,17}(-[a-z][a-z0-9]{0,17})?$</code><br/>
        </td>
    </tr>
</table>
