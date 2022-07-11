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

    fetch(runUri).then(resp => resp.text()).then(text => {
        output_inner.innerHTML = text;
    });
}

function likeSnippet() {
    fetch(likeUri);
}
