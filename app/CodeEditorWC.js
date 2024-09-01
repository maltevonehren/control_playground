import {EditorState}
    from "@codemirror/state"
import {EditorView, gutter, lineNumbers, keymap, drawSelection}
    from "@codemirror/view"
import {defaultKeymap, indentWithTab}
    from "@codemirror/commands"

class CodeEditorWC extends HTMLElement {

    constructor() {
        super();
        this.view = null;
    }

    async connectedCallback() {
        this.style.display = 'block'
        if (!this.style.width) { this.style.width = '100%' }
        if (!this.style.height) { this.style.height = '100%' }

        const listener = EditorView.updateListener.of((v) => {
            if(v.docChanged) {
                this.dispatchEvent(new Event("myInput"));
            }
        });
        
        const src = this.getAttribute('initialSrc');
        const state = EditorState.create({
            doc: src,
            extensions: [
                keymap.of(defaultKeymap),
                keymap.of([indentWithTab]),
                lineNumbers(),
                gutter(),
                EditorState.allowMultipleSelections.of(true),
                drawSelection(),
                EditorView.clickAddsSelectionRange.of(e => e.altKey),
                listener,
            ],
        });
        this.view = new EditorView({
            state,
            parent: this,
        })
    }

    get value() {
        return this.view.state.doc.toString();
    }
}

customElements.define('code-editor', CodeEditorWC);
