# Design

## Conventions

Items that have a definition in the Trifonius Glossary are named as a capitalized noun
(possibly pre- or post-fixed with an adjective noun, separated by a space)
and are styled in italics. Examples are

* _Processor_
* _Processor Realization_
* _Inbound Junction_

Items that appear in the API or implementation of Trifonius are named as a capitalized noun
(possibly pre- or post-fixed with an adjective noun, without a space)
and are styled in a fixed with font. Examples are

* `ProcessorId`
* `ProcessorRealizationId`
* `ProcessorTechnology`

Identifiers

* `Id` - Identifier of a item/component/junction in the scope of a Trifonius component.
  The scope of an `Identifier` is never global.
* `Identifier` - Identifier of a item/component/junction with a global scope.
  An `Identifier` is always a composite value, typically constructed from the defining scope
  (or scopes) and the target item/component, separated by a dot (`.`).
* `Label` - Human readable label of an item or parameter, used in the interaction with the
  pipeline designer. Labels are typically defined by the Trifonius framework or
  in the _Processor Realization_ configuration files.
* `Name` - Human readable name of an item or component, specified by the pipeline designer.

