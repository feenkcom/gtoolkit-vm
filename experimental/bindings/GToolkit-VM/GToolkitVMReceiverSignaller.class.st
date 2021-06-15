Class {
	#name : #GToolkitVMReceiverSignaller,
	#superclass : #FFIExternalObject,
	#instVars : [
		'thunk'
	],
	#category : #'GToolkit-VM'
}

{ #category : #printing }
GToolkitVMReceiverSignaller >> printOn: aStream [

	"Append a sequence of characters to aStream that identify the receiver."

	super printOn: aStream.
	aStream
		nextPutAll: ' handle: ';
		print: handle.
	aStream
		nextPutAll: ' thunk: ';
		print: thunk
]

{ #category : #accessing }
GToolkitVMReceiverSignaller >> thunk [

	^ thunk
]

{ #category : #accessing }
GToolkitVMReceiverSignaller >> thunk: anObject [

	thunk := anObject
]
