let scripts = document.getElementsByTagName('script');
let this_script = scripts[scripts.length - 1];
let word = this_script.getAttribute('data-word');

let appendEl = function (into, tag, { id = '', clazz = '', onclick = '', style = '', inner = '', source = '' } = {}) {
    into.appendChild(createEl(tag, { id, clazz, onclick, style, inner, source }));
}

let createEl = function (tag, { id = '', clazz = '', onclick = '', style = '', inner = '', source = '' } = {}) {
    let el = document.createElement(tag);
    if (id)
        el.setAttribute('id', id);
    if (clazz)
        el.className = clazz;
    if (onclick)
        el.setAttribute('onclick', onclick);
    if (style)
        el.setAttribute('style', style);
    if (inner)
        el.innerHTML = inner;
    if (source)
        el.setAttribute('src', source);
    return el;
}

let openDict = function (evt, dictName) {
    if (document.getElementById(dictName).style.display === 'block')
        return;
    var dicts = document.getElementsByClassName('dictionary');
    for (let dict of dicts) {
        dict.style.display = "none";
        dict.className = dict.className.replace(' fade-in', '');
    }
    var tabs = document.getElementsByClassName('bar-item');
    for (let tab of tabs) {
        tab.className = tab.className.replace(' bar-selected', '');
    }
    document.getElementById(dictName).style.display = 'block';
    document.getElementById(dictName).className += ' fade-in';
    evt.currentTarget.className += ' bar-selected';
}

let start_time = Date.now();

fetch('/api/define/' + word).then(function (response) {
    return response.json();
}).then(function (data) {
    console.log('fetched in ' + (Date.now() - start_time) / 1000 + 's');

    console.log(data);

    let word_container = $('#word')[0];

    if (Object.keys(data).length === 0) {
        let div = createEl('div');
        appendEl(div, "h3", { inner: "couldn't find that word", clazz: 'gray', style: 'padding: 25% 0' });
        appendEl(div, 'p', { inner: 'Try removing endings such as -ed or -s,<br>or try changing the capitilazation' });
        appendEl(div, 'p', { inner: 'Phrases might be harder to find, and there\'s no autocorrect yet' });
        word_container.appendChild(div);
        word_container.className += 'fade-in';
        return;
    }

    let word_left = createEl('div', { clazz: 'word-column' });
    let word_right = createEl('div', { clazz: 'word-column' });

    if (data['overview']) {
        let overview = createEl('div', { id: 'word-overview' });

        let tag = 'h3';
        for (line of data['overview']) {
            appendEl(overview, tag, { inner: line })
            tag = 'h4';
        }

        word_left.appendChild(overview);
    }

    let tab_div = createEl('div', { clazz: 'tab-bar' });

    let first_sel = ' bar-selected';
    let add_tab = function (title, id, data) {
        if (!data || data.length === 0)
            return;
        appendEl(tab_div, 'button', { clazz: 'bar-item' + first_sel, inner: title, onclick: `openDict(event, '${id}')` });
        if (first_sel) {
            first_sel = '';
        }
    }

    add_tab('Macmillan', 'Macmillan', data['macmillan_defs']);
    add_tab('Vocabulary.com', 'Vocab', data['vocab_defs']);
    add_tab('Wikitionary', 'Wiki', data['wiki_defs']);
    add_tab('Urban Dictionary', 'Urban', data['urban_defs']);

    word_left.appendChild(tab_div);

    let first = true;
    let add_definitions = function (name, definitions) {
        if (!definitions || definitions.length === 0)
            return;

        let defs_div = createEl('ul', { id: name, clazz: 'dictionary' });
        if (!first) {
            defs_div.setAttribute('style', 'display:none')
        } else {
            defs_div.setAttribute('style', 'display:block')
            first = false;
        }

        let entries = [];
        let last_pos = '';

        for (let def of definitions) {
            let appendSense = function (sense, li_clazz) {
                if (entries.length === 0 || last_pos !== sense['part_of_speech']) {
                    entries.push(createEl('ol', { clazz: 'word-entry' }));
                    if (entries.length !== 0)
                        appendEl(entries[entries.length - 1], 'br')
                    appendEl(entries[entries.length - 1], 'span', {
                        clazz: 'part-of-speech ' + sense['part_of_speech'],
                        inner: sense['part_of_speech']
                    });
                    last_pos = sense['part_of_speech'];
                }

                let li = createEl('li', { clazz: li_clazz });

                let def_content = createEl('div', { clazz: 'def-content' });

                appendEl(def_content, 'span', { clazz: 'meaning', inner: sense['meaning'] })

                li.appendChild(def_content);

                let examples = createEl('ul', { clazz: 'examples' });
                for (let example of sense['examples']) {
                    appendEl(examples, 'li', { clazz: 'example', inner: example })
                }
                li.appendChild(examples);

                return li;
            };

            if (Array.isArray(def)) {
                let first = appendSense(def[0], 'numbered');
                let subsenses = createEl('ol');
                def.shift()
                for (let subsense of def) {
                    let sub_li = appendSense(subsense, 'lettered');
                    subsenses.appendChild(sub_li);
                }
                first.appendChild(subsenses);
                entries[entries.length - 1].appendChild(first);
            } else {
                let first = appendSense(def, 'numbered');
                entries[entries.length - 1].appendChild(first);
            }
        }

        for (entry of entries) {
            defs_div.appendChild(entry);
        }

        word_left.appendChild(defs_div);
    }

    add_definitions('Macmillan', data['macmillan_defs']);
    add_definitions('Vocab', data['vocab_defs']);
    add_definitions('Wiki', data['wiki_defs']);
    add_definitions('Urban', data['urban_defs']);

    // now the right side

    if (data['stock_images']) {
        let img_container = createEl('div', { clazz: 'img-container' });
        for (idx in data['stock_images']) {
            appendEl(img_container, 'img', {
                clazz: 'stock-img',
                source: data['stock_images'][idx]
            });
        }
        word_right.appendChild(img_container);
    }

    let origin_div = createEl('ul', { clazz: 'origins' });

    let add_origins = function (origins) {
        if (!origins)
            return;
        for (let origin of origins) {
            let li = createEl('li', { class: 'origin' });

            if (origin['part_of_speech']) {
                appendEl(li, 'span', {
                    clazz: 'part-of-speech ' + origin['part_of_speech'],
                    inner: origin['part_of_speech']
                });
            }

            let paras = origin['origin'].split('<br>');

            for (let para of paras) {
                appendEl(li, 'p', { clazz: 'origin-text', inner: para });
            }

            origin_div.appendChild(li);
        }
    }

    if (data['etym_origins']) {
        appendEl(origin_div, 'h2', { inner: 'Word Origin' });
        add_origins(data['etym_origins']);
    } else if (data['wiki_origins']) {
        appendEl(origin_div, 'h2', { inner: 'Word Origin' });
        add_origins(data['wiki_origins']);
    }

    word_right.appendChild(origin_div);

    let sources = createEl('div', { clazz: 'sources' });
    appendEl(sources, 'h4', { clazz: 'fancy', inner: 'Not the right word?' })
    appendEl(sources, 'p', { inner: 'Try removing endings such as -ed or -s,<br>or try changing the capitilazation' });
    appendEl(sources, 'p', { inner: 'Phrases might be harder to find, and there\'s no autocorrect yet' });
    appendEl(sources, 'br');
    appendEl(sources, 'h4', { clazz: 'fancy', inner: 'Sources' })
    for (let source of data['sources']) {
        appendEl(sources, 'p', { inner: source });
    }
    appendEl(sources, 'br');
    appendEl(sources, 'p', { inner: 'Have a suggestion? Send it to <a href="https://github.com/NotAFlyingGoose/" target="_blank" rel="noopener noreferrer">NotAFlyingGoose</a>' });
    word_right.appendChild(sources);

    word_container.appendChild(word_left);
    word_container.appendChild(word_right);

    word_container.className += 'slide-up';

    console.log('finished in ' + (Date.now() - start_time) / 1000 + 's');
}).catch(function (err) {
    console.log('Fetch Error :-S', err);
});