<?php
$CONFIG = array(
    'memcache.local' => '\\OC\\Memcache\\APCu',
    'apps_paths' =>
    array(
        0 =>
        array(
            'path' => '/var/www/html/apps',
            'url' => '/apps',
            'writable' => false,
        ),
        1 =>
        array(
            'path' => '/var/www/html/custom_apps',
            'url' => '/custom_apps',
            'writable' => true,
        ),
    ),
    'instanceid' => 'ocahihcdz0lk',
    'passwordsalt' => getenv('PASSWORD_SALT'),
    'secret' => getenv('ENCRYPTION_SECRET'),
    'trusted_domains' =>
    array(
        0 => 'clic.epfl.ch',
    ),
    'datadirectory' => '/var/www/html/data',
    'dbtype' => 'mysql',
    'version' => '27.0.1.2',
    'default_phone_region' => 'CH',
    'dbname' => 'nextcloud',
    'dbhost' => 'database',
    'dbport' => '',
    'dbtableprefix' => 'oc_',
    'mysql.utf8mb4' => true,
    'dbuser' => 'nextcloud',
    'dbpassword' => '1234',
    'installed' => true,
    'theme' => '',
    'loglevel' => 3,
    'maintenance' => false,
    'overwriteprotocol' => 'https',
    'overwritehost' => 'clic.epfl.ch',
    'overwritewebroot' => '/nextcloud',
    'htaccess.RewriteBase' => '/',
    'mail_smtpmode' => 'smtp',
    'mail_sendmailmode' => 'smtp',
    'mail_from_address' => 'it.clic',
    'mail_domain' => 'epfl.ch',
    'mail_smtphost' => 'mail.epfl.ch',
    'mail_smtpport' => '587',
    'auth.bruteforce.protection.enabled' => false,
    'mail_smtpsecure' => 'tls',
    'mail_smtpauthtype' => 'LOGIN',
    'mail_smtpauth' => 1,
    'mail_smtpname' => 'it.clic',
    'mail_smtppassword' => getenv('STMP_PASSWORD'),
    'twofactor_enforced' => 'true',
    'twofactor_enforced_groups' =>
    array(
        0 => 'admin',
    ),
    'twofactor_enforced_excluded_groups' =>
    array(),
);
