# pdf-viewer
Centralized PDF reading from your browser.

 ## why?
 I wanted a centralized way of reading course literature and other nice things in PDF format from anywhere which also synced my progress. 
 Instead of actually reading course literature I threw together this piece of software. 
 
 ## No, I meant why is it so bad??
 So this was just meant to be a internal tool for reading things, if I find areas to improve (there definetly are some of those) I will try push those changes throught the fall.

## How do I use it?
in almost all scenarios you are better off buying a kindle or using some proprietary PDF reader but if the concept tempts you it is just a 
```bash
$ cargo install pdf-viewer
``` 
away!
By default it will read PDFs from the `content` directory but this can be changed (see `pdf-viewer --help`)
