/* eslint-disable */

var overlay = document.querySelector('.bg-overlay');
var container = document.querySelector('.container');
var style_span = document.querySelector('#style-name');
var ext_span = document.querySelector('#ext');
var lines_span = document.querySelector('#lines');
var highlighted = document.querySelector('#highlighted');
var style_select = document.querySelector('#style');
var filetype_select = document.querySelector('#filetype');
var highlight_label = document.querySelector('#highlight+label');

var highlighted_lines = {
  js:   '11-20',
  jsx:  '1,22-33',
  html: '3-10',
  rs:   '8,19-23',
  py:   '8,15-17',
  c:    '8-11',
  go:   '7-15',
};

var style = 'github';
var filetype = 'rs';

function isLight(color) {
  var rgb = color.match(/\d+/g);
  var r = parseInt(rgb[0], 10);
  var g = parseInt(rgb[1], 10);
  var b = parseInt(rgb[2], 10);

  var luma = 0.2126 * r + 0.7152 * g + 0.0722 * b; // per ITU-R BT.709

  return (luma > 170);
}

function alpha(color, a) {
  return color.replace('rgb', 'rgba').replace(')', `, ${a})`);
}

document.getElementById('style').onchange = function(e) {
  style = e.target.value;

  var selector = `.P--${style}`;
  var styles = window.getComputedStyle(document.querySelector(selector));
  var fg = styles.color;
  var bg = styles.backgroundColor;

  if (isLight(bg)) {
    document.body.classList.remove('dark');
  } else {
    document.body.classList.add('dark');
  }

  container.style.color = fg;
  overlay.style.backgroundColor = bg;

  style_select.style.backgroundColor = bg;
  style_select.style.borderColor = alpha(fg, 0.25);
  filetype_select.style.backgroundColor = bg;
  filetype_select.style.borderColor = alpha(fg, 0.25);
  highlight_label.style.backgroundColor = bg;
  highlight_label.style.borderColor = alpha(fg, 0.25);

  highlighted.dataset.style = `${style}--${filetype}`;
  style_span.innerHTML = style.replace(/_/g, ' ');

  updateCheckbox(document.getElementById('highlight').checked);
};

document.getElementById('filetype').onchange = function(e) {
  filetype = e.target.value;
  highlighted.dataset.style = `${style}--${filetype}`;
  highlighted.dataset.filetype = filetype;
  ext_span.innerHTML = filetype;

  updateCheckbox(document.getElementById('highlight').checked);
};

document.getElementById('highlight').onchange = function(e) {
  updateCheckbox(e.target.checked);
};

function updateCheckbox(isChecked) {
  if (isChecked) {
    highlighted.className = '';

    var sel = `.P--${style} .hi`;
    var bg = window.getComputedStyle(document.querySelector(sel)).backgroundColor;
    var lines = highlighted_lines[highlighted.dataset.filetype];

    highlight_label.style.backgroundColor = bg;
    lines_span.innerHTML = `--highlight=${lines}`;
  } else {
    var sel = `.P--${style}`;
    var bg = window.getComputedStyle(document.querySelector(sel)).backgroundColor;

    highlighted.className = 'no-hi';
    highlight_label.style.backgroundColor = bg;
    lines_span.innerHTML = '';
  }
}
