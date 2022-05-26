let btn = document.querySelector("#run-btn");
let output = document.querySelector("#run-output");
let output_inner = document.querySelector("#run-output code");

// Add random colored background to comments
document.querySelectorAll(".comment").forEach((c) => {
    let hue = Math.random() * 360;
    c.style.background = "hsl(" + hue + ", 60%, 88%)";
});

function runCode() {
    output.style.display = "block";
    output_inner.innerHTML = "Loading...";
    
    let xmlHttp = new XMLHttpRequest();
    xmlHttp.onreadystatechange = function() { 
        if (xmlHttp.readyState == 4 && xmlHttp.status == 200) {
            output_inner.innerHTML = xmlHttp.responseText;
        }
    }

    xmlHttp.open("GET", runUri, true);
    xmlHttp.send(null);
}
