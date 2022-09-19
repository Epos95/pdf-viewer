# pdf-viewer
Centralized PDF reading from your browser.

 ## why?
 I wanted a centralized way of reading course literature and other nice things in PDF format from anywhere which also synced my progress. 
 Instead of actually reading course literature I threw together this piece of software. 
 
 ## No, I meant why doesnt it do `<FEATURE HERE>`???
 So this was just meant to be a internal tool for reading things, if I find areas to improve (there definetly are some of those) I will try push those changes through out the fall. (or you can just put on your big-person pants and put in a PR)

## How do I use it?
in almost all scenarios you are better off buying a kindle or using some proprietary PDF reader but if the concept tempts you, then you probably know enough to build it from source :)

## What does it even do?
So this program simply exposes all the PDF files in the given directory (see `-h` option for more info) for anyone to view on the specified port.
The program then keeps track of where you are in each PDF so that you can seamlessly transition from reading on your laptop to your phone or desktop and vice versa. In case of a desync (the page you try to turn to is not the "next page" as signified by the server's state) the website will ask you if you want to continue on the local page or jump to the one stored on the server.
