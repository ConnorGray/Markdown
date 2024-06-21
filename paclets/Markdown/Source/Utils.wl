BeginPackage["ConnorGray`Markdown`Utils`"]

NotebookImportCell

Begin["`Private`"]

Needs["ConnorGray`Markdown`"]
Needs["Wolfram`ErrorTools`"]

(*====================================*)

SetFallthroughError[NotebookImportCell]

NotebookImportCell[cell_CellObject] :=
	NotebookImportCell[NotebookRead[cell]]

NotebookImportCell[cell_Cell, form_?StringQ] := Module[{
	result
},
	result = RaiseConfirmMatch[
		First[
			UsingFrontEnd @ NotebookImport[
				Notebook[{cell}],
				_ -> form
			],
			Missing["NotAvailable"]
		],
		_?StringQ | _?MissingQ
	];

	result
]

End[]

EndPackage[]