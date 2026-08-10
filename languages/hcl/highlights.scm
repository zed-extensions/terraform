; https://github.com/nvim-treesitter/nvim-treesitter/blob/cb79d2446196d25607eb1d982c96939abdf67b8e/queries/hcl/highlights.scm
; highlights.scm
[
  "!"
  "\*"
  "/"
  "%"
  "\+"
  "-"
  ">"
  ">="
  "<"
  "<="
  "=="
  "!="
  "&&"
  "||"
] @operator

[
  "{"
  "}"
  "["
  "]"
  "("
  ")"
] @punctuation.bracket

[
  "."
  ".*"
  ","
  "[*]"
] @punctuation.delimiter

[
  (ellipsis)
  "\?"
  "=>"
] @punctuation.special

[
  ":"
  "="
] @punctuation

[
  "for"
  "endfor"
  "in"
  "if"
  "else"
  "endif"
] @keyword

[
  (quoted_template_start) ; "
  (quoted_template_end) ; "
  (template_literal) ; non-interpolation/directive content
] @string

[
  (heredoc_identifier) ; END
  (heredoc_start) ; << or <<-
] @punctuation.delimiter

[
  (template_interpolation_start) ; ${
  (template_interpolation_end) ; }
  (template_directive_start) ; %{
  (template_directive_end) ; }
  (strip_marker) ; ~
] @punctuation.special

(numeric_lit) @number

(bool_lit) @boolean

(null_lit) @constant

(comment) @comment

(identifier) @variable

(body
  (block
    (identifier) @keyword))

(body
  (block
    (body
      (block
        (identifier) @type))))

(function_call
  (identifier) @function)

(attribute
  (identifier) @variable)

; { key: val }
;
; highlight identifier keys as though they were block attributes
(object_elem
  key: (expression
    (variable_expr
      (identifier) @variable)))

; var.foo, data.bar
;
; The first step (`var`, `local`, `module`...) stays neutral, every
; following step is highlighted, so that the meaningful part of a
; reference stands out from its namespace.
; @property is the conventional tree-sitter capture for a member access.
(_
  (variable_expr
    (identifier) @variable)
  (get_attr
    (identifier) @property))
