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

fetch('/api/' + word).then(function (response) {
    return response.json();
}).then(function (data) {
    console.log('fetched in ' + (Date.now() - start_time) / 1000 + 's');

    console.log(data);

    let word_container = $('#word')[0];

    if (Object.keys(data).length === 0) {
        appendEl(word_container, "h3", { inner: "couldn't find that word", clazz: 'gray' });
        word_container.className += 'fade-in';
        return;
    }

    let word_left = createEl('div', { clazz: 'word-column' });
    let word_right = createEl('div', { clazz: 'word-column' });

    if (data['overview']) {
        let overview = createEl('div', { id: 'wordOverview' });

        let tag = 'h3';
        for (raw_line of data['overview']) {
            appendEl(overview, tag, { inner: decodeURIComponent(raw_line) })
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
            if (entries.length === 0 || last_pos !== def['part_of_speech']) {
                entries.push(createEl('ol', { clazz: 'word-entry' }));
                if (entries.length !== 0)
                    appendEl(entries[entries.length - 1], 'br')
                appendEl(entries[entries.length - 1], 'span', {
                    clazz: 'part_of_speech ' + def['part_of_speech'],
                    inner: def['part_of_speech']
                });
                last_pos = def['part_of_speech'];
            }

            let li = createEl('li', { clazz: 'numbered' });

            let def_content = createEl('div', { clazz: 'defContent' });

            appendEl(def_content, 'span', { clazz: 'meaning', inner: decodeURIComponent(def['meaning']) })

            li.appendChild(def_content);

            let examples = createEl('ul', { clazz: 'examples' });
            for (let example of def['examples']) {
                appendEl(examples, 'li', { clazz: 'example', inner: decodeURIComponent(example) })
            }
            li.appendChild(examples);

            entries[entries.length - 1].appendChild(li);
        }

        for (entry of entries) {
            defs_div.appendChild(entry);
        }

        word_left.appendChild(defs_div);
    }

    add_definitions('Vocab', data['vocab_defs']);
    add_definitions('Wiki', data['wiki_defs']);
    add_definitions('Urban', data['urban_defs']);

    // now the right side

    if (data['stock_images']) {
        let img_containers = [
            createEl('div', { clazz: 'img-container' }),
            createEl('div', { clazz: 'img-container' }),
        ];
        for (idx in data['stock_images']) {
            appendEl(img_containers[Math.floor(idx / 3)], 'img', {
                clazz: 'stock-img',
                source: data['stock_images'][idx]
            });
        }
        for (container of img_containers) {
            word_right.appendChild(container);
        }
    }

    let origin_div = createEl('ul', { clazz: 'origins' });
    appendEl(origin_div, 'h2', { inner: 'Word Origin' });

    let add_origins = function (origins) {
        if (!origins)
            return;
        for (let origin of origins) {
            let li = createEl('li', { class: 'origin' });

            if (origin['part_of_speech']) {
                appendEl(li, 'span', {
                    clazz: 'part_of_speech ' + origin['part_of_speech'],
                    inner: origin['part_of_speech']
                });
            }

            let raw_origin = decodeURIComponent(origin['origin']);
            let paras = raw_origin.split('<br>');

            for (let para of paras) {
                appendEl(li, 'p', { clazz: 'origin-text', inner: para });
            }

            origin_div.appendChild(li);
        }
    }

    add_origins(data['etym_origins']);
    if (!data['etym_origins']) {
        // appendEl(origin_div, 'h2', 'Wiktionary', { clazz: 'alternate-small' });
        add_origins(data['wiki_origins']);
    }

    word_right.appendChild(origin_div);

    word_left.innerHTML += `<script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pub-5229740966840782" crossorigin="anonymous"><\/script><!-- Left --><ins class="adsbygoogle" style="display:block" data-ad-client="ca-pub-5229740966840782" data-ad-slot="3795749516" data-ad-format="auto" data-full-width-responsive="true"><\/ins><script>(adsbygoogle = window.adsbygoogle || []).push({});<\/script>`;
    word_right.innerHTML += `<script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pub-5229740966840782" crossorigin="anonymous"><\/script><ins class="adsbygoogle" style="display:block;" data-ad-client="ca-pub-5229740966840782"data-ad-slot="8201898383" data-ad-format="auto" data-full-width-responsive="true"><\/ins><script>(adsbygoogle = window.adsbygoogle || []).push({});<\/script>`;

    let sources = createEl('div', { clazz: 'sources' });
    appendEl(sources, 'h4', { inner: 'Sources' })
    for (let source of data['sources']) {
        appendEl(sources, 'p', { inner: source });
    }
    appendEl(sources, 'br');
    appendEl(sources, 'br');
    appendEl(sources, 'p', { inner: 'This site is kept up with your ads. Thank You' });
    appendEl(sources, 'br');
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