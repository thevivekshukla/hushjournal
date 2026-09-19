ALTER TABLE journals
    ADD COLUMN theme TEXT NOT NULL DEFAULT '';

ALTER TABLE journals
    ADD CONSTRAINT journals_theme_values CHECK (
        theme IN (
            '',
            'silk',
            'dim',
            'light',
            'dark',
            'cupcake',
            'bumblebee',
            'emerald',
            'corporate',
            'synthwave',
            'retro',
            'cyberpunk',
            'valentine',
            'halloween',
            'garden',
            'forest',
            'aqua',
            'lofi',
            'pastel',
            'fantasy',
            'wireframe',
            'black',
            'luxury',
            'dracula',
            'cmyk',
            'autumn',
            'business',
            'acid',
            'lemonade',
            'night',
            'coffee',
            'winter',
            'nord',
            'sunset',
            'caramellatte',
            'abyss'
        )
    );
