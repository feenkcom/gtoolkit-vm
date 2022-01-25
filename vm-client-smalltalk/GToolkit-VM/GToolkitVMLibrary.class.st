Class {
	#name : #GToolkitVMLibrary,
	#superclass : #FFILibrary,
	#category : #'GToolkit-VM'
}

{ #category : #resources }
GToolkitVMLibrary class >> ffiLibrary [
	^ self
]

{ #category : #converting }
GToolkitVMLibrary >> calloutAPIClass [
	^ FFIBackend current calloutAPIClass
]

{ #category : #'accessing platform' }
GToolkitVMLibrary >> macModuleName [
	^ self moduleName
]

{ #category : #'accessing platform' }
GToolkitVMLibrary >> moduleName [
	^ FileLocator vmBinary asFileReference fullName
]

{ #category : #accessing }
GToolkitVMLibrary >> runner [
	^ self subclassResponsibility
]

{ #category : #'accessing platform' }
GToolkitVMLibrary >> unixModuleName [
	^ self moduleName
]

{ #category : #'accessing platform' }
GToolkitVMLibrary >> win32ModuleName [
	^ self moduleName
]
