## Identfiers

* ProcessorIdentifier
* ResourceIdentifier
* JunctionIdentifier

<table>
    <tr style="vertical-align: top;">
        <th>identifier</th>
        <th>regular expression</th>
        <th>description</th>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>DshServiceName</code></td>
        <td style="white-space:nowrap;"><code>^[a-z][a-z0-9]{0,19}-[a-z][a-z0-9]{0,19}$</code></td>
        <td>
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>JunctionId</code></td>
        <td><code>^[a-z][a-z0-9_-]{1,50}$</code></td>
        <td>
            A <code>JunctionId</code> identifies an inbound or outbound junction in a processor. 
            <code>JunctionId</code>s must be unique within the scope of their containing processor, 
            so an inbound junction can not have the same id 
            as an outbound junction.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ParameterId</code></td>
        <td><code>^[a-z][a-z0-9_-]{1,30}$</code></td>
        <td>
            A <code>ParameterId</code> identifies a parameter that must be provided
            by the client of the engine.
            Parameters (and hence the <code>ParameterId</code>s) are defined in the 
            processor configuration and can be retrieved from the backend as part 
            of the processor descriptor. When the client wants to deploy a pipeline
            <code>ParameterId</code>s must be unique within the scope of their 
            containing processor.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>PipelineId</code></td>
        <td><code>^[a-z][a-z0-9]{0,19}$</code></td>
        <td>
            A <code>PipelineId</code> identifies a declared and saved pipeline 
            within Trifonius.
            A <code>PipelineId</code> is created by the Trifonius backend 
            when a new profile is received from the client, after which it cannot be changed.
            The <code>PipelineId</code> should also be used to store the layout 
            of the profile in de backend.
            <br/>
            The <code>PipelineId</code> could, together with the 
            <code>ServiceId</code>, possibly be used to generate 
            the name of a deployed service on the DSH (<code>DshServiceName</code>).
            Therefor it needs to adhere to strict rules regarding length and syntax.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProcessorId</code></td>
        <td><code>^[a-z][a-z0-9]{0,19}$</code></td>
        <td>
            A <code>ProcessorId</code> identifies a declared and saved processor 
            component in the scope of a declared and saved pipeline.
            A <code>ProcessorId</code> is declared when created by the Trifonius backend 
            when a new profile is received from the client, 
            but from then on it should also be used to store the layout 
            of the profile in de backend.
            <br/>
            The <code>ServiceId</code> could, together with the 
            <code>ProfileId</code>, possibly be used to generate 
            the name of a deployed service on the DSH (<code>DshServiceName</code>).
            Therefor it needs to adhere to strict rules regarding length and syntax.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ProfileId</code></td>
        <td><code>^[a-z0-9]{1,20}$</code></td>
        <td>
            A <code>ProfileId</code> identifies a profile declared within the scope a processor. 
            <code>ProfileId</code>s must be unique within the scope of their containing processor.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ResourceId</code></td>
        <td><code>^[a-z][a-z0-9_-]{1,50}$</code></td>
        <td>
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>ServiceId</code></td>
        <td><code>^[a-z][a-z0-9]{0,19}$</code></td>
        <td>
            A <code>ServiceId</code> identifies a declared and saved processor 
            component in a pipeline within Trifonius.
            A <code>PipelineId</code> is created by the Trifonius backend 
            when a new profile is received from the client, 
            but from then on it should also be used to store the layout 
            of the profile in de backend.
            <br/>
            The <code>ServiceId</code> could, together with the 
            <code>ProfileId</code>, possibly be used to generate 
            the name of a deployed service on the DSH (<code>DshServiceName</code>).
            Therefor it needs to adhere to strict rules regarding length and syntax.
        </td>
    </tr>
    <tr style="vertical-align: top;">
        <td><code>TaskId</code></td>
        <td><code>^.*$</code></td>
        <td>
        </td>
    </tr>
</table>


