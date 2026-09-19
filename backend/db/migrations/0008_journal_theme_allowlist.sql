UPDATE journals
SET theme = ''
WHERE theme <> ALL (
    ARRAY[
        'silk',
        'cupcake',
        'bumblebee',
        'emerald',
        'corporate',
        'nord',
        'lemonade',
        'winter',
        'caramellatte',
        'retro',
        'dim',
        'forest',
        'dracula',
        'night',
        'coffee',
        'synthwave',
        'abyss',
        'luxury',
        'halloween',
        'sunset'
    ]
);

ALTER TABLE journals
    DROP CONSTRAINT journals_theme_values;

ALTER TABLE journals
    ADD CONSTRAINT journals_theme_values CHECK (
        theme IN (
            '',
            'silk',
            'cupcake',
            'bumblebee',
            'emerald',
            'corporate',
            'nord',
            'lemonade',
            'winter',
            'caramellatte',
            'retro',
            'dim',
            'forest',
            'dracula',
            'night',
            'coffee',
            'synthwave',
            'abyss',
            'luxury',
            'halloween',
            'sunset'
        )
    );
