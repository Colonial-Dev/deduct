.PRECIOUS: %.tex

objects := $(patsubst texf/%.texf,%.svg,$(wildcard texf/*.texf))

all: $(objects)
	sed -i 's/rgb(0%, 0%, 0%)/rgb(100%, 100%, 100%)/g' *.svg

%.tex: texf/%.texf texf/rule-template.tex forallxyyc.sty
	pandoc -f latex -t latex --template texf/rule-template.tex -o $@ $<

%.pdf: %.tex
	pdflatex $*

%.svg: %.pdf
	pdf2svg $< $@

clean:
	rm -f *.tex *.pdf *.aux *.out *.log
