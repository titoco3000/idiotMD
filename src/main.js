const { invoke } = window.__TAURI__.tauri;

function renderMath() {
  for(let element of document.getElementsByClassName("math-part")){
    jqMath.parseMath(element);
  }
}

function debug(something){
  
  if(document.getElementById("debug") == null){
    document.innerHTML= '<p id="debug"></p>'+document.innerHTML;
  }
  document.getElementById("debug").innerText = something;
}

window.addEventListener("DOMContentLoaded", () => {
  let mainInput = document.getElementById("main-input");
  let resultPlace = document.getElementById("result");

  invoke("openinitialfile").then((result)=>{
    mainInput.value = result[0];
    resultPlace.innerHTML = result[1];
  });
  
  mainInput.addEventListener("input",()=>{
    invoke("parse", { input: mainInput.value}).then((result)=>{
      resultPlace.innerHTML = result;
      renderMath();
    })
  });
  
  document.addEventListener('keydown', function(event) {
    if (event.ctrlKey) {
      if(event.shiftKey){
        //debug('shift');
        if(event.key === 'S'){
          //debug('shift+s');
          invoke("savefile", { content: mainInput.value, forceDialog: true });
        }
      } 
      else if(event.key === 'o'){
        invoke("openfiledialog").then((result)=>{
          if(result[0]!="" && result[1]!=""){
            mainInput.value = result[0];
            resultPlace.innerHTML = result[1];
          }
        });
      }
      else if(event.key === 's'){
        invoke("savefile", { content: mainInput.value });
      }
      else if(event.key === 'n'){
        invoke("newfile");
        mainInput.value = "";
        resultPlace.innerHTML = "";
      }
    }
  });

  // register('CmdOrControl+O', () => {
  //   console.log('crtl o');
  //   invoke("openfiledialog").then((result)=>{
  //     if(result[0]!="" && result[1]!=""){
  //       mainInput.innerHTML = result[0];
  //       resultPlace.innerHTML = result[1];
  //     }
  //   });
  // });
  
  // register('CmdOrControl+S', () => {
  //   console.log('crtl s');
  //   invoke("savefiledialog", { content: mainInput.value });
  // });

});
