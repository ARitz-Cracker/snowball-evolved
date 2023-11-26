# CEWT

_Custom elements with (type-guarded) templates. Pronouced like "cute"_

The scope of this project is currently in flux. There are some remnants of incomplete code back when this project was intended to eventually be an SSR/SSG suite, but then I realized, "that's needlessly complicated"! I don't imagine the TypeScript code generation will change much (if at all) so if you'd like to use this as part of your project, it's probably best to stick with only the `cewt codegen` command with the `--inline-html` option.

## Installing

I currently don't provide any pre-compiled binaries, so for now, you can `cargo install cewt`.

## Overview

Cewt doesn't intend to force you to subscribe to an entirely new abstraction (read: lie and burden the user with the consequences) on how you interact with the DOM. The use of auto-generated TypeScript code from HTML snippets has 2 key benefits.

1. Less computational overhead at runtime
2. Is completely transparent about the work it tries to do for you.

### TypeScript code generation

When defining your elements, `cewt` recursively searches for `.html` files in a specified folder. The `.html` files it looks for are those which contains one or more `<template>` elements defined at the root level. These templates can be used help create _Autonomous Custom Elements_ and _Customized Built-in Elements_. 

#### Autonomous Custom Elements

* Extends `HTMLElement`
* Are shown in the document as (for example) `<my-custom-element>`
    * Therefore query-selectable as `my-custom-element`
* Utilizes your template snippet by cloning the inner elements to a Shadow-DOM
    * Allows for the use of `<slot>` elements to help facilitate SSR
    * Is kind-of its own document tree, which means query-selecting the document cant't touch the elements defined from the template.
    * Is kind-of its own CSS context, allowing you to link CSS files without affecting the entire document, though the document's CSS rules may still affect your element, for example, inherenting font styles.

#### Customized Built-in Elements

* Extend the class of whatever element you wish to expend
* Are shown in the document as (for example) `<div is="my-custom-element">`
    * Therefore query-selectable as `div[is="my-custom-element"]`
* Utilizes your template snippet by cloning the inner elements as children of the custom element, though only if the element is empty when it is retrieved from the server.
* Useful if you just want an element with some custom code associated with an HTML running without changing the semantics of your document.

Note: At the time of writing, Safari currently can't do _Customized Built-in Elements_, in Apple's [typical "think different" fasion](https://github.com/WebKit/standards-positions/issues/97). Cewt currently doesn't provide any polyfill's on its own. My personal solution is to optionally import the `@ungap/custom-elements` npm package if the following check fails. 

```ts
function supportsExtendingBuiltinElements() {
    try {
        const newElemName = "test-button-" + Date.now().toString(36);
        class HTMLTestButton extends HTMLButtonElement {};
        customElements.define(newElemName, HTMLTestButton, { extends: "button" });
        const newBtn = document.createElement("button", { is: newElemName });
        return newBtn instanceof HTMLButtonElement && newBtn instanceof HTMLTestButton;
    }catch(ex: any) {
        return false;
    }
}
```

You will have to do this if you want to utilize _Customized Built-in Elements_ while having support for Safari. (which is the _only_ browser on iPhone's btw, as on iOS, Apple only gives you the illusion of choice) If your project only uses _Autonomous Custom Elements_, or you don't care about iPhone's, then the polyfill won't be necessary. If you do use the polyfill, remember to import it _**BEFORE**_ you register your custom elements.

## How to use

Using cewt has 2 steps.

1. Defining your template
2. Extending upon the auto-generated code
3. Generating HTML
    * Not always required. I might remove this step anyway.

### Template definitions

To define your custom elements using Cewt, use the `cewt codegen` command.
* This command has 1 required parameter
    * `<PATH>`, a folder containing HTML templates which may be in sub-folders.
* This command also has 2 optional parameters
    * `--exclude <NAME>`, (can be specified multiple times) specifying folder names to exclude from the recursive file search search, defaulting to `node_modules`.
    * `--inline-html`, which will include your template snippet's inner HTML as part of the auto-generated code.
        * If not specified, by default, the auto-generated code will reference a `<template>` by a unique auto-generated ID. Though this will require the "bundle" step as shown below.
        * You can specify this option if
            * You have no use of inline templates (templates with no code)
            * You want your custom element to be a part of a portable module
            * Including your template HTML snippets for every generated full *.html file seems stupid to you.

The placement of the auto-generated TypeScript file depends on whether or not your HTML snippet files where named `template.html`, or similar to `my-element-thing.html`. If your HTML snippet file was named `template.html`, it will create an `_autogen.ts` in the same folder as the HTML file, else it will create an `_autogen` folder if it didn't exist, and create a `.ts` file with the same base-name as the `.html` file. For example:

* (...)
    * my-component
        * _autogen.ts (Don't touch)
        * template.html (contains 1 or more templates)
        * index.ts (your code here, file can be named whatever)
    * my-other-component
        * _autogen.ts (Don't touch)
        * template.html (contains 1 or more templates)
        * index.ts (your code here, file can be named whatever)
* (...)
    * _autogen (Don't touch)
        * my-component.ts
        * my-other-component.ts
    * my-component.html
    * my-component.ts
    * my-other-component.html
    * my-other-component.ts

Note that you can have multiple template definitons per HTML snippet file. All auto-generated classes will just be in the same resulting `.ts` file.

### Basic example

Here's an example of a basic _Autonomous Custom Element_ definition.

```html
<template cewt-name="cewt-intro">
    <h1>The Custom Element</h1>
    <p>Says: <slot name="intro-text"></slot></p>
</template>
```

Note that when generating the TypeScript code, it checks the first child element of `<slot>` element to determine which elements will fill the slot. If none are found, it assumes the slot will be filled in by a `<span>`. Of course, there are no real guarantees that `slots.introText` is an `HTMLSpanElement`, but TypeScript being a real language is just a lie we tell ourselves to make our IDE's autocomplete slightly more aware of what our code is supposed to do anyway, and it's why I choose to use the term "type-guarded" instead of "type-safe" when describing this tool.

Assuming the file was named `template.html`, we can create the following TypeScript.

```ts
import {CewtIntroAutogen} from "./_autogen.ts";

class CewtIntroElement extends CewtIntroAutogen {
    constructor() {
        super();
        // Custom behaviour here if desired.
        // Note that the element might not be in the document at this point.
        // This is called after registration if the element was already on the page.
    }
    connectedCallback() {
        // Called whenever the element has been added to the page.
        // Also called after registration if the element was already on the page.
        // It is recommended to move have logic here as elements might be constructed only to be thrown away.
        // Though also note elementes removed from the page may also be re-added.

        // All properties of `this.slots` are getter functions. If an element doesn't exist, it is created.
        console.log("<cewt-intro> has been added to the page, and it says: " + this.slots.introText.innerText);
    }
    disconnectedCallback() {
        // cleanup logic, etc.
        console.log("<cewt-intro> has been removed from the page");
    }
    adoptedCallback() {
        // Not needed in most cases, see https://stackoverflow.com/questions/50995139/ for details.
    }
}
CewtIntroElement.registerElement(); // Required. Make sure this is called with the child class.
```

Now, when the following is in the HTML document sent to the user (assuming your code is also imported)

```html
<cewt-intro>
    <span slot="intro-text">This is the first example!</span>
</cewt-intro>
```

The following will be printed to console

```
<cewt-intro> has been added to the page, and it says: This is the first example!
```

To create this element programatically, you can simply `new CewtIntroElement()`, for some reason `document.createElement("cewt-intro")` doesn't work in all browsers.

#### Custom attributes example

If you want your custom element to have custom attributes, you can define them like so
```html
<template cewt-name="example-element" cewt-attributes="my-attribute, my-other-attribute">
</template>
```

Which allows you to use them like so

```ts
import {ExampleElementAutogen} from "./_autogen.ts";

class ExampleElementElement extends ExampleElementAutogen {
    constructor() {
        super();
    }
    onMyAttributeChanged(oldValue: string | null, newValue: string | null) {
        // called when the function name implies
    }
    onMyOtherAttributeChanged(oldValue: string | null, newValue: string | null) {
        // called when the function name implies
    }
    connectedCallback() {
        // The super-class includes getters/setters for your custom attributes
        console.log("my-attribute is:", this.myAttribute);
        console.log("my-other-attribute is:", this.myOtherAttribute);
    }
}
ExampleElementElement.registerElement(); // Required. Make sure this is called with the child class.
```

Which can be referenced in the document like so

```html
<example-element my-attribute="value" my-other-attribute="valueeee"></example-element>
```

#### Extending build-in elements

Despite the Webkit team's opinions on the matter, extending buildin elements is actually useful, because you can create `<button>` elements or `<dialog>` elements with custom-defined behaviour that exists in a scoped context, i.e., your extended class. [See also \"Drawbacks of autonomous custom elements\" by WHATWG](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-elements-autonomous-drawbacks)

Anyway, here's an example.

```html
<template cewt-name="counter-example" cewt-extends="button">
    You've clicked me <span cewt-ref="count">0</span> time(s)!
</template>
```

```ts
class CounterExampleElement extends CounterExampleAutogen {
    count: number;
	constructor() {
		super();
		this.count = 0;
		this.addEventListener("click", (ev) => {
			this.count += 1;
            // Refs are just references to elements that aren't a part of the slot system
			this.refs.count.innerText = this.count;
		})
	}
}
CounterExampleElement.registerElement();
```

Since this extends `<button>` you can tab-focus and enter-click to your heart's content. Just add `<button is="counter-example"></button>` to the document or construct a `new CounterExampleElement()` to add it programmatically.

### HTML document generation

This step is required if
* You have inline templates
* You did not specify `--inline-html` during typescript code generation

I'm debating whether or not HTML document generation is out of scope for this tool, so I won't write detailed documentation on it, but for now, you can use run `cewt bundle-single --help` to get started.
