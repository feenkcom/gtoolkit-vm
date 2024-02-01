Class {
	#name : #BaselineOfGToolkitVMMaker,
	#superclass : #BaselineOf,
	#classVars : [
		'PostLoaded'
	],
	#category : #BaselineOfGToolkitVMMaker
}

{ #category : #accessing }
BaselineOfGToolkitVMMaker class >> loadPackage: aPackage fromRepo: repo [
	| reader |
	reader := TonelReader on: repo / 'pharo-vm' / 'smalltalksrc' fileName: aPackage.
	reader loadPackage.
	reader loadDefinitions.
	EpMonitor disableDuring: [ reader snapshot install ]
]

{ #category : #accessing }
BaselineOfGToolkitVMMaker class >> loadVMMaker: aDirectory [
	"Manually installs VMMaker from the sources in gtoolkit-vm/pharo-vm"
	| repo |

	repo := aDirectory asFileReference.
	(GtExternalProcessBuilder new: 'git')
		workingDirectory: repo;
		arguments: {'submodule'.
				'update'.
				'--init'.
				'--recursive'};
		status.

	{'CAST'.
		'Slang'.
		'Slang-Tests'.
		'Printf'.
		'Melchor'.
		'VMMakerLoadingDependencies'.
		'VMMakerCompatibilityForPharo6'}
		do: [ :eachPackage | self loadPackage: eachPackage fromRepo: repo ].

	[ self loadPackage: 'VMMaker' fromRepo: repo ]
		on: Warning
		do: [ :e | e resume ].

	(self class environment at: #StackInterpreter)
		initializeBytecodeTableForSistaV1
]

{ #category : #baseline }
BaselineOfGToolkitVMMaker >> baseline: spec [
	<baseline>
		^ spec for: #common do: [
			spec postLoadDoIt: #loadVMMaker: ]
]

{ #category : #accessing }
BaselineOfGToolkitVMMaker >> loadVMMaker: spec [
	(PostLoaded isNotNil and: [ PostLoaded ]) ifTrue: [ ^ self ].

	self class loadVMMaker: (FileLocator localDirectory / 'iceberg' / 'feenkcom' / 'gtoolkit-vm')
			resolve.
	
	PostLoaded := true
]
