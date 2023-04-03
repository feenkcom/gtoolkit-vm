(RBAddPragmaTransformation pragma: '<api>' inMethod: #instantiateClass:isPinned: inClass: #SpurMemoryManager)
    asRefactoring execute.


SpurMemoryManager compile: 'getStatFullGCUsecs
    <api>
    ^ statFullGCUsecs'.

SpurMemoryManager compile: 'getStatScavengeGCUsecs
    <api>
    ^ statScavengeGCUsecs'.

Smalltalk snapshot: true andQuit: true
