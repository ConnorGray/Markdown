BeginPackage["ConnorGray`Markdown`ConvertToMarkdown`"]

Begin["`Private`"]

Needs["Wolfram`ErrorTools`"]

Needs["ConnorGray`Markdown`"]
Needs["ConnorGray`Markdown`Utils`"]

(*========================================================*)

SetFallthroughError[CreateMarkdownNotebook]

CreateMarkdownNotebook[] := Module[{

},
	NotebookPut @ Notebook[
		{},
		StyleDefinitions -> FrontEnd`FileName[
			{"ConnorGray"},
			"Markdown.nb",
			CharacterEncoding -> "UTF-8"
		]
	]
]

CreateMarkdownNotebook[markdown0_?StringQ] := Module[{
	markdown = MarkdownParse[markdown0]
},
	(* TODO: Improve error handling, if this can even error *)
	RaiseAssert[MatchQ[markdown, {___MarkdownElement}]];
]

(*========================================================*)

SetFallthroughError[ConvertToMarkdownElement]

ConvertToMarkdownElement[obj_] := Module[{},
	ConfirmReplace[obj, {
		(*==================*)
		(* FrontEnd Objects *)
		(*==================*)

		(* TODO: Add tests for each of these cases. *)

		_NotebookObject :> ConvertToMarkdownElement[NotebookGet[obj]],

		_CellObject :> ConvertToMarkdownElement[NotebookGet[obj]],

		(*======================*)
		(* Notebook Expressions *)
		(*======================*)

		Notebook[cells_List, options___?OptionQ] :> Map[ConvertToMarkdownElement, cells],

		cell_Cell :> convertToMarkdown[cell],

		other_ :> Raise[
			MarkdownError,
			"Unrecognized form cannot be converted to Markdown: ``",
			InputForm[other]
		]
	}]
]

(*====================================*)

$trivialCellOpts = OrderlessPatternSequence[
	RepeatedNull[CellChangeTimes -> _, 1],
	RepeatedNull[CellLabel -> _, 1]
];

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

	cell:Cell[_, "Input", $trivialCellOpts] :> Module[{
		inputCode
	},
		inputCode = RaiseConfirm @ NotebookImportCell[cell, "InputText"];

		MarkdownElement["CodeBlock", "wolfram,cell:Input", inputCode]
	],

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

	TextData[contents_?ListQ] :> (
		(* FIXME:
			Should not always be a single paragraph if `contents` contains
			consecutive internal newlines or other formatting constructs.
		*)
		Splice @ Map[convertToMarkdown, contents]
	),

	StyleBox[expr1_, styles___?OptionQ] :> Module[{
		expr
	},
		expr = Fold[
			{expr2, style} |-> ConfirmReplace[style, {
				HoldPattern[FontWeight -> "Bold"] :> (
					MarkdownElement["Strong", {expr2}]
				),
				HoldPattern[FontSlant -> "Italic"] :> (
					MarkdownElement["Emphasis", {expr2}]
				),
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
			convertToMarkdown[expr1],
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