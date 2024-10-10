# Technologies, realizations and identifiers

## Glossary

<table>
    <tr style="vertical-align: top;">
        <th>term</th>
        <th>description</th>
        <th>unique id</th>
        <th>examples</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Pipeline</em></td>
        <td>
            A Trifonius <em>Pipeline</em> is a set of collaborating processors and/or 
            resources, together with a specification how these components are 
            connected together and how they are configured.
            A deployed and started pipeline can realize a requested capability.
        </td>
        <td><code>PipelineId</code> + <code>Version</code></td>
        <td><code>itv-pipeline</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Pipeline Profile</em></td>
        <td>
            A <em>Pipeline Profile</em> is a set of parameters that can alter 
            the runtime behavior of a pipeline without having to specify a new pipeline.
            Typically the pipeline profile consists of the processor profiles and 
            resource profiles of the constituent processor and resource instances.
        </td>
        <td><code>ProfileId</code></td>
        <td><code>log-level</code><br/><code>medium</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Processor Technology</em></td>
        <td>
            A <em>Processor Technology</em> is a technical solution
            for the realization of a Trifonius processor.
            Examples are DSH services or apps from the DSH App Catalog.
            Support for more technology solutions will be added later,
            e.g. Flink, Polars, Nifi, et cetera.
        </td>
        <td><code>ProcessorTechnology</code></td>
        <td><code>dshapp</code><br/><code>dshservice</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Processor Realization</em></td>
        <td>
            A <em>Processor Realization</em> is one technical processor component 
            that is available for use by Trifonius, 
            and is based on one of the processor technologies
            (e.g. a container in the DSH Registry). 
            A processor realization is usually implemented by leveraging 
            an (existing or specially built)
            technical component based on the processor technology and 
            by specifying its behavior, characteristics and deployment requirements
            in a configuration file.
            The available processor realizations will show up in the processor catalog or registry.
        </td>
        <td><code>ProcessorRealizationId</code><br/>+ <code>Version</code></td>
        <td><code>replicator</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Processor Instance</em></td>
        <td>
            A <em>Processor Instance</em> is a configured processor realization 
            used in a pipeline.
            In order to use the processor realization, 
            the pipeline must assign the instance a <code>ProcessorId</code>,
            which must be unique in the scope of the pipeline. 
            The pipeline must also specify all the deployment requirements 
            of the processor realization, and provide a human friendly name.
        </td>
        <td><code>ProcessorId</code></td>
        <td><code>itv-replicator</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Processor Profile</em></td>
        <td>
            A <em>Processor Profile</em> is a set of parameters that can alter 
            the runtime behavior of a processor without having to specify a new processor.
        </td>
        <td><code>ProfileId</code></td>
        <td><code>log-level</code><br/><code>medium</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Resource Technology</em></td>
        <td>
            A <em>Resource Technology</em> is a technical solution
            which implements a source/sink resource that is available to the Trifonius 
            processors. 
            At this time the only available resource technology 
            are the Kafka topics managed by the DSH.
            Support for more technology solutions will be added later,
            e.g. S3 buckets, databases, web-services, et cetera.
        </td>
        <td><code>ResourceTechnology</code></td>
        <td><code>dshtopic</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Resource Realization</em></td>
        <td>
            A <em>Resource Realization</em> is a source/sink resource 
            that is available for use by Trifonius, 
            and will show up in the resource catalog or registry.
            A resource realization is always implemented using a technical solution 
            based on one of the resource technologies, 
            e.g. a Kafka topic managed by DSH.
            The set of available resource realizations typically 
            originates outside Trifonius, and is defined by the current installed base 
            of the technical resource. 
            E.g., the set of resources for the technology resource 
            <code>dshtopic</code> consists of the set of Kafka topics accessible to the tenant.
        </td>
        <td><code>ResourceRealizationId</code><br/> + <code>Version</code></td>
        <td><code>stb-status</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Resource Instance</em></td>
        <td>
            A <em>Resource Instance</em> is a configured resource realization 
            used in a pipeline.
            In order to use the resource realization, 
            the pipeline must assign the instance a <code>ResourceId</code>,
            which must be unique in the scope of the pipeline.
            The pipeline must also specify all the deployment requirements 
            of the resource realization (if any), and provide a human friendly name.
        </td>
        <td><code>ResourceId</code></td>
        <td><code>status-topic</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><em>Resource Profile</em></td>
        <td>
            A <em>Resource Profile</em> is a set of parameters that can alter 
            the runtime behavior of a resource without having to specify a new resource.
            Note that since most resources will be not managed by Trifonius, 
            specifying a resource profile often makes no sense.
        </td>
        <td><code>ProfileId</code></td>
        <td><code>log-level</code></td>
    </tr>
</table>

## Ids

* Pipeline ids
    * `PipelineId` - Uniquely identifies a pipeline
      (together with a <code>Version</code>).
    * `Version` - Specifies a certain version of a pipeline.
    * `ProfileId` - Specifies the profile that is used when a pipeline
      is deployed or started.
* Processor ids
    * `ProcessorTechnology` - Identifies the technical solutions available
      for the realization of processors.
    * `ProcessorRealizationId` - Identifies a specific realization of a processor,
      based on a specific processor technology, usually with a configuration file.
    * `Version` - Specifies a certain version of a processor realization.
    * `JunctionId` - Identifies the junctions in a processor realization,
      that connect a processor to resources or other processors.
    * `ParameterId` - Identifies the parameters that the pipeline designer must provide
      when using a processor in a pipeline.
    * `ProcessorId` - Identifies an instance of a processor realization
      in the scope of a pipeline.
    * `ProfileId` - Specifies the profile that is used when a processor
      is deployed or started.
* Resource ids
    * `ResourceTechnology` - Identifies the technical solutions that realize resources.
    * `ResourceRealizationId` - Identifies a specific realization of a resource,
      based on a specific resource technology.
    * `Version` - Specifies a certain version of a resource realization.
      A version is mandatory, although in some cases it doesn't make sense
      (e.g. for DSH Kafka topics). In these cases `0.0.0` must be used.
    * `ResourceId` - Identifies an instance of a resource realization
      in the scope of a pipeline.
    * `ProfileId` - Specifies the profile that is used when a resource
      is deployed or started.
      Optional, because in many cases a profile doesn't make sense as resources
      are managed outside Trifonius (e.g. for DSH Kafka topics).

## Pipeline ids

### `PipelineId` + `Version`

A `PipelineId` identifies a declared and saved pipeline
as it will be managed by Trifonius.
A `Version` identifies the version of a pipeline.
Since the `PipelineId` is the primary key of a pipeline in the
pipeline registry, it cannot be changed once it is defined.
However, for the representation of a pipeline to the pipeline designer or user
via the frontend,
a more user-friendly name can be used.
This name can be changed after it was defined (although this might be confusing for the designer).
The `PipelineId` will be generated by the backend when a
pipeline is first
declared/saved. The backend will attempt to generate a more or less readable name,
based on the user-friendly name that was provided with the first declaration.
The `PipelineId`, together with the `Version`, will also be used to store the layout
of the pipeline in de layout-backend.

Note that for some processor technologies (`dshapp` and `dshservice`) the `PipelineId`
will be used to construct the name of a deployed service on the DSH (together with a
`ProcessorId`), yielding a `PipelineProcessorId`.
Therefor both the `PipelineId` and the `ProcessorId` need to adhere to strict rules
regarding length and syntax.

### `ProfileId`

A pipeline's `ProfileId` identifies a deployment profile which enables to provide some
parameters of a pipeline (and its constituent processors and resources) at runtime,
without having to define an entirely new pipeline.

## Processor ids

### `ProcessorTechnology`

A `ProcessorTechnology` defines one technical solution
for the realization of Trifonius processors.
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
        <td><code>dshapp</code></td>
        <td>
          DSH App processors make Apps published to the DSH App Catalog eligible
          for deployment and control via Trifonius. This type is not yet supported, 
          but is planned for the near future.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>dshservice</code></td>
        <td>
          DSH Service processors make containers published in the DSH container registry 
          (Harbor) eligible for deployment and control via Trifonius.
        </td>
    </tr>
</table>

### `ProcessorRealizationId` + `Version`

`ProcessorRealizationId`s define the processors that are available to Trifonius designers.
They typically consist of a component realized using one of the processor technologies,
together with a configuration file to define its behavior within Trifonius.
The configurations will be read by the Trifonius engine at application startup time or dynamic.
Adding a new processor requires writing a new configuration file,
that makes an already available or specially built DSH service or app eligible for
deployment and control via Trifonius.
Building these DSH services and apps, and pushing/publishing them to the
container registry or app catalog is more or less independent of Trifonius.
Defining a new processor realization will typically not require any changes to the engine,
backend or frontend.

### `JunctionId`

`JunctionId`s identify inbound or outbound junctions in the scope
of a processor realization.
`JunctionId`s must be unique within the scope of their containing
processor realization, so an inbound junction can not have the same
`JunctionId` as an outbound junction.

### `ParameterId`

A `ParameterId` identifies a parameter that must be provided
by the pipeline designer when he is designing a pipeline.
A parameter (and hence `ParameterId`) is defined in the
processor realization configuration file and can be retrieved from the backend as part
of the processor descriptor.
`ParameterId`s must be unique within the scope of their containing processor realization.

### `ProcessorId`

A `ProcessorId` uniquely identifies a processor instance within the scope
of a pipeline, as it will be managed by Trifonius.
A `ProcessorId` identifies a declared and saved processor realization
component in the scope of a declared and saved pipeline.
The `ProcessorId` must be defined by the Trifonius pipeline designer as part of the design.
The `ProcessorId` will also be used as a subkey to store the position
of the processor in de layout-backend.

Note that for some processor technologies (`dshapp` and `dshservice`) the `ProcessorId`
will be used to construct the name of a deployed service on the DSH (together with a
`PipelineId`), yielding a `PipelineProcessorId`.
Therefor the `ProcessorId` needs to adhere to strict rules regarding length and syntax.

### `ProfileId`

A processor's `ProfileId` identifies a deployment profile which enables to provide some
parameters of a processor at runtime,
without having to define an entirely new pipeline.

## Resource ids

### `ResourceTechnology`

A `ResourceTechnology` defines one technical solution
for the realization of a Trifonius source or sink resource.
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
        <td><code>dshgateway</code></td>
        <td>
          DSH gateway resources make DSH stream topics, connected to the DSH gateway,
          eligible for use and control via Trifonius. This type is not yet supported, 
          but is planned for the near future.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>dshtopic</code></td>
        <td>
          DSH topic resources make ordinary DSH Kafka topics eligible for use and 
          control via Trifonius.
        </td>
    </tr>
</table>

### `ResourceRealizationId` + `Version`

The set of available resource realizations, and hence the list of `ResourceRealizationId`s,
typically originates outside Trifonius, and is defined by the current installed base
of the technical resource.
E.g., the set of resources for the technology resource
<code>dshtopic</code> consists of the set of Kafka topics accessible to the tenant.
Therefor the list of `ResourceRealizationId`s will be derived from the list of available topics,
making sure that unicity and the syntactical requirements are met. In this case a
resource version does not make sense, so `0.0.0` will be used for the `Version`.
Adding new resource realizations will typically not require any changes to the engine,
backend or frontend.

### `ResourceId`

A `ResourceId` identifies a resource within the scope
of a pipeline, as it will be managed by Trifonius.
A `ResourceId` can be defined by the Trifonius pipeline designer.
The `ResourceId` will also be used as a subkey to store the position
of the resource in de layout-backend.

Note that in the future there might be new resource technologies that will be
implemented as services on the DSH.
In that case it will be necessary to construct the name of a deployed service on the DSH from a
`ResourceId` together with a `PipelineId`, yielding a `PipelineResourceId`.
Therefor a `ResourceId` needs to adhere to the same strict syntactical rules as a `ProcessorId`.

## Identifiers

While the ids describes above identify their components within the scope of their
containing components, there are also some identifiers that are used to identify
processors and resource in the scope of a defined pipeline or their respective registries.
E.g., a `ProcessorIdentifier` like `(dshservice, greenbox-consent-filter)` can be used to
reference a specific processor realization when used in a pipeline, or requested from the registry.

* `ProcessorIdentifier` - Uniquely identifies a processor realization
  when instantiated in a pipeline.
* `ResourceIdentifier` - Uniquely identifies a resource realization
  when instantiated in a pipeline.

### `ProcessorIdentifier`

A `ProcessorIdentifier` is a tuple consisting of a `ProcessorTechnology`,
a `ProcessorRealizationId` and a `Version`. It can be used to reference a
processor realization when instantiating it in a pipeline or requesting it from the registry.

### `ResourceIdentifier`

A `ResourceIdentifier` is a tuple consisting of a `ResourceTechnology`,
a `ResourceRealizationId` and a `Version`). It can be used to reference a
resource realization when instantiating it in a pipeline or requesting it from the registry.

## Syntax

Some ids or identifier are used to construct other (composite) identifiers,
which possibly must adhere to some restrictions.
E.g. a `PipelineId` and a `ProcessorId` will be used to construct the name of a DSH service,
which can be at most 45 characters long and may contain only
ASCII alphabetical characters and digits.
Therefor there are some strict syntactical restrictions on the ids.

<table>
    <tr style="vertical-align: top;">
        <th>identifier</th>
        <th>length</th>
        <th>syntax / regex</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>JunctionId</code></td>
        <td><code>1 - 50</code></td>
        <td><code>^[a-z][a-z0-9-]{0,49}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ParameterId</code></td>
        <td><code>1 - 50</code></td>
        <td><code>^[a-z][a-z0-9-]{0,49}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineId</code></td>
        <td><code>1 - 18</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineProcessorId</code></td>
        <td><code>3 - 37</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}-[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineResourceId</code></td>
        <td><code>3 - 37</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}-[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorId</code></td>
        <td><code>1 - 18</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorIdentifier</code></td>
        <td></td>
        <td><code>(ProcessorTechnology, ProcessorRealizationId, Version)</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorRealizationId</code></td>
        <td><code>1 - 50</code></td>
        <td><code>^[a-z][a-z0-9-]{0,49}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorTechnology</code></td>
        <td><code>1 - 10</code></td>
        <td><code>^[a-z0-9]{0,9}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProfileId</code></td>
        <td><code>1 - 50</code></td>
        <td><code>^[a-z][a-z0-9-]{0,49}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceId</code></td>
        <td><code>1 - 18</code></td>
        <td><code>^[a-z][a-z0-9]{0,17}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceIdentifier</code></td>
        <td></td>
        <td><code>(ResourceTechnology, ResourceRealizationId, Version?)</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceRealizationId</code></td>
        <td><code>1 - 50</code></td>
        <td><code>^[a-z][a-z0-9-]{0,49}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceTechnology</code></td>
        <td><code>1 - 10</code></td>
        <td><code>^[a-z][a-z0-9]{0,9}$</code></td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>Version</code></td>
        <td></td>
        <td><code>(major, minor, patch)</code></td>
    </tr>
</table>
