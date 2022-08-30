const token = "{{token}}";
console.log("token: " + token);
var url = "http://localhost:3000/get_pdf/"+pdf_name;

// Loaded via <script> tag, create shortcut to access PDF.js exports.
var pdfjsLib = window['pdfjs-dist/build/pdf'];

// The workerSrc property shall be specified.
pdfjsLib.GlobalWorkerOptions.workerSrc = '//mozilla.github.io/pdf.js/build/pdf.worker.js';

var pdfDoc = null;
var pageNum = parseInt(window.pdf_page);
var pdf_name = window.pdf_name;
var pageRendering = false;
var pageNumPending = null;
var scale = 1.2;
var canvas = document.getElementById('the-canvas');
var ctx = canvas.getContext('2d');

/**
* Get page info from document, resize canvas accordingly, and render page.
* @param num Page number.
*/
function renderPage(num) {
    pageRendering = true;
    // Using promise to fetch the page
    pdfDoc.getPage(num).then(function(page) {
        var viewport = page.getViewport({scale: scale});
        canvas.height = viewport.height;
        canvas.width = viewport.width;

        // Render PDF page into canvas context
        var renderContext = {
            canvasContext: ctx,
            viewport: viewport
        };
        var renderTask = page.render(renderContext);

        // Wait for rendering to finish
        renderTask.promise.then(function() {
            pageRendering = false;
            if (pageNumPending !== null) {
                // New page rendering is pending
                renderPage(pageNumPending);
                pageNumPending = null;
            }
        });
    });

    // Update page counters
    document.getElementById('page_num').textContent = num;
}

/**
* If another page rendering in progress, waits until the rendering is
* finised. Otherwise, executes rendering immediately.
*/
function queueRenderPage(num) {
    if (pageRendering) {
        pageNumPending = num;
    } else {
        renderPage(num);
    }
}

/**
* Displays previous page.
*/
function onPrevPage() {
    if (pageNum <= 1) {
        return;
    }

    set_page("-");
}
document.getElementById('prev').addEventListener('click', onPrevPage);

/**
* Displays next page.
*/
function onNextPage() {
    if (pageNum >= pdfDoc.numPages) {
        return;
    }

    set_page("+");
}
document.getElementById('next').addEventListener('click', onNextPage);

function set_page(direction) {
    var dest = "http://" + window.location.host + "/status/"+pdf_name;
    fetch(dest).then(function(response) {
        return response.json();
    }).then(function(data) {
        server_page = parseInt(data);
        console.log(server_page);

        if (server_page === NaN || server_page == -1) {
            // return early on error
            return;
        }

        if (pageNum != server_page) {
            if (confirm("Desynced!\nJump to the page stored remotely?\n(local is at page: " + pageNum + ", server is at page: " + server_page + ")")) {
                pageNum = data;
            }
        }

        if (direction == "+") {
            pageNum++;
        } else {
            pageNum--;
        }

        var dest = "http://"+window.location.host+"/view/"+pdf_name+"/set_page";
        fetch(dest, {
            method: "POST",
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({
                "token" : "",
                "pdf_name" : pdf_name,
                "new_page" : pageNum,
            })
        }).then(res => {});
        queueRenderPage(pageNum);
    }).catch((e) => {
        console.log("Booo");
        console.log(e);
    });

}

/**
* Asynchronously downloads PDF.
*/
pdfjsLib.getDocument(url).promise.then(function(pdfDoc_) {
    pdfDoc = pdfDoc_;
    document.getElementById('page_count').textContent = pdfDoc.numPages;

    // Initial/first page rendering
    renderPage(pageNum);
});

document.onkeydown = checkKey;
function checkKey(e) {
    e = e || window.event;

    if (e.keyCode == '37') {
        onPrevPage();
    }
    else if (e.keyCode == '39') {
        onNextPage();
    }

}
