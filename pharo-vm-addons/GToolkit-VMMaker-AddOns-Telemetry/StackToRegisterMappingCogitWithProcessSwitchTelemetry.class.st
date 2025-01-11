Class {
	#name : #StackToRegisterMappingCogitWithProcessSwitchTelemetry,
	#superclass : #StackToRegisterMappingCogit,
	#category : #'GToolkit-VMMaker-AddOns-Telemetry'
}

{ #category : #translation }
StackToRegisterMappingCogitWithProcessSwitchTelemetry class >> declareCVarsIn: aCCodeGenerator [
	aCCodeGenerator
		addHeaderFileFirst: '"telemetry.h"'
]
