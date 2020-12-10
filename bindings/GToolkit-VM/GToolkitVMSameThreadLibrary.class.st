"
! Same thread library
Use me as `FFILibrary` if you want to query ${class:GToolkitVM}$ from the same thread in which the VM is running
"
Class {
	#name : #GToolkitVMSameThreadLibrary,
	#superclass : #GToolkitVMLibrary,
	#category : #'GToolkit-VM'
}

{ #category : #accessing }
GToolkitVMSameThreadLibrary >> runner [
	^ TFSameThreadRunner uniqueInstance
]
