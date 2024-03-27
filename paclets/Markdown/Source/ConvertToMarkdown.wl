BeginPackage["ConnorGray`Markdown`ConvertToMarkdown`"]

Begin["`Private`"]

Needs["Wolfram`ErrorTools`"]

Needs["ConnorGray`Markdown`"]

(*========================================================*)

SetFallthroughError[CreateMarkdownNotebook]

CreateMarkdownNotebook[markdown0_?StringQ] := Module[{
	markdown = MarkdownParse[markdown0]
},
	(* TODO: Improve error handling, if this can even error *)
	RaiseAssert[MatchQ[markdown, {___MarkdownElement}]];
]

(*========================================================*)

SetFallthroughError[ConvertToMarkdown]

ConvertToMarkdown[obj_] := Module[{},
	ConfirmReplace[obj, {
		(*==================*)
		(* FrontEnd Objects *)
		(*==================*)

		(* TODO: Add tests for each of these cases. *)

		_NotebookObject :> ConvertToMarkdown[NotebookGet[obj]],

		_CellObject :> ConvertToMarkdown[NotebookGet[obj]],

		(*======================*)
		(* Notebook Expressions *)
		(*======================*)

		Notebook[cells_List, options___?OptionQ] :> Map[ConvertToMarkdown, cells],

		cell_Cell :> convertToMarkdown[cell],

		other_ :> Raise[
			MarkdownError,
			"Unrecognized form cannot be converted to Markdown: ``",
			InputForm[other]
		]
	}]
]

(*====================================*)

$trivialCellOpts = OrderlessPatternSequence[]

SetFallthroughError[convertToMarkdown]

convertToMarkdown[expr0_] := ConfirmReplace[expr0, {
	(* FIXME: I'm sure this isn't correct in all cases. *)
	text_?StringQ :> MarkdownElement["Text", text],

	(*================================*)
	(* Known Cell Styles              *)
	(*================================*)

	(* TODO:
		This should require that cdata convert to only Inline markdown
	    elements. *)
	Cell[cdata_, "Title", $trivialCellOpts] :> (
		MarkdownElement["Heading", 1, {requireInlines @ convertToMarkdown[cdata]}]
	),

	Cell[cdata_, "Section", $trivialCellOpts] :> (
		MarkdownElement["Heading", 3, {requireInlines @ convertToMarkdown[cdata]}]
	),

	Cell[cdata_, "Text", $trivialCellOpts] :> (
		MarkdownElement["Paragraph", {requireInlines @ convertToMarkdown[cdata]}]
	),

	Cell[cdata_?StringQ, "Program", $trivialCellOpts] :> (
		MarkdownElement["CodeBlock", None, cdata]
	),

	cell_Cell :> (
		Raise[
			MarkdownError,
			<| "Cell" -> cell |>,
			"Cell with unrecognized form cannot be converted to Markdown."
		]
	),

	(*================================*)
	(* Box Data                       *)
	(*================================*)

	StyleBox[expr1_, styles___?OptionQ] :> Module[{
		expr
	},
		expr = Fold[
			{expr2, style} |-> ConfirmReplace[style, {
				HoldPattern[lhs_ -> rhs_] :> (
					Raise[
						MarkdownError,
						"Unrecognized StyleBox rule cannot be converted to Markdown: `` -> ``",
						InputForm[lhs],
						InputForm[rhs]
					]
				),
				other_ :> Raise[
					MarkdownError,
					"Unsupported StyleBox style: ``",
					InputForm[other]
				]
			}],
			expr1,
			{styles}
		];

		expr
	],

	other_ :> Raise[
		MarkdownError,
		"Unsupported form cannot be converted to Markdown: ``",
		InputForm[other]
	]
}]

(*====================================*)

(* FIXME:
	This should raise a descriptive exception if `expr` contains
	non-Inlines markdown content. At the moment this is just a marker.
*)
SetFallthroughError[requireInlines]

requireInlines[expr_] := expr

End[]

EndPackage[]