(RBAddPragmaTransformation pragma: '<api>' inMethod: #instantiateClass:isPinned: inClass: #SpurMemoryManager)
    asRefactoring execute.


SpurMemoryManager compile: 'getStatFullGCUsecs
    <api>
    ^ statFullGCUsecs'.

SpurMemoryManager compile: 'getStatScavengeGCUsecs
    <api>
    ^ statScavengeGCUsecs'.

(AbstractImageAccess class >> #implicitReturnTypeFor:) removeFromSystem.

Smalltalk snapshot: true andQuit: true