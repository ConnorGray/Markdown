BeginPackage["ConnorGray`Markdown`MarkdownToNotebook`"]

Begin["`Private`"]

Needs["Wolfram`ErrorTools`"]

Needs["ConnorGray`Markdown`"]

(*========================================================*)

SetFallthroughError[MarkdownToNotebook]

MarkdownToNotebook[markdown0_] := WrapRaised[
	MarkdownError,
	"Error converting Markdown to notebook"
] @ Module[{
	cells
},
	cells = {markdownToCells[markdown0]};

	RaiseAssert[MatchQ[cells, {___Cell}]];

	Notebook[
		cells,
		StyleDefinitions -> FrontEnd`FileName[
			{"ConnorGray"},
			"Markdown.nb",
			CharacterEncoding -> "UTF-8"
		]
	]
]

(*========================================================*)

SetFallthroughError[markdownToCells]

(*
	markdownToCells[expr] converts a MarkdownElement or {___MarkdownElement}
	expression into notebook Cell content.

	The returned cells expression will always be in one of the following
	forms:

	* Cell[...]
	* Splice[{___Cell}]
	* Nothing
*)

markdownToCells[markdown0_] := ConfirmReplace[markdown0, {
	list_?ListQ :> (
		Splice @ Map[markdownToCells, list]
	),

	MarkdownElement["Heading", level_?IntegerQ, inlines_List] :> Module[{
		style
	},
		style = ConfirmReplace[level, {
			1 -> "Title",
			(* TODO: This style isn't used often anymore. *)
			2 -> "Chapter",
			3 -> "Section",
			4 -> "Subsection",
			5 -> "Subsubsection",
			6 -> "Subsubsubsection",
			other_ :> Raise[
				MarkdownError,
				"Invalid heading level: ``",
				InputForm[other]
			]
		}];

		Cell[
			TextData[{inlinesToTextData[inlines]}],
			style
		]
	],

	MarkdownElement["Paragraph", inlines_List] :> (
		Cell[
			TextData[{inlinesToTextData[inlines]}],
			"Text"
		]
	),

	MarkdownElement["CodeBlock", infoString_?StringQ, program_?StringQ] :> (
		Cell[
			program,
			"Program",
			TaggingRules -> <|
				"ConnorGray/Markdown" -> <|
					"CodeBlockInfoString" -> infoString
				|>
			|>
		]
	),

	(*================================*)
	(* Unrecognized Markdown form     *)
	(*================================*)

	other_ :> Raise[
		MarkdownError,
		<| "MarkdownExpression" -> other |>,
		"Unrecognized Markdown form"
	]
}]

(*====================================*)

SetFallthroughError[inlinesToTextData]

inlinesToTextData[inlines0_] := ConfirmReplace[inlines0, {
	list_?ListQ :> Splice @ Map[inlinesToTextData, list],

	MarkdownElement["Text", text_?StringQ] :> text,

	other_ :> Raise[
		MarkdownError,
		<| "Expression" -> other |>,
		"Not a recognized Markdown inline element."
	]
}]

(*========================================================*)

End[]

EndPackage[]