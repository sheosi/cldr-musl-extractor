use std::{collections::HashMap, path::PathBuf};

use anyhow::{Result, anyhow};

use crate::model::ModelMessages;

/// Returns the messages saved for a language, if the file doesn't exist, returns
/// the default en_US messages, if there's any other error it is returned.
pub async fn parse_messages(lang: &str, region: &str) -> Result<Option<ModelMessages>> {
    let path = PathBuf::from(format!("musl-locales/{lang}_{region}"));
    if !path.exists() {
        println!("INFO: Musl locale not found for {lang}_{region}");
        return Ok(None);
    };

    let file_contents = tokio::fs::read_to_string(path).await?;
    parse_messages_of_content(&file_contents).map(Option::Some)
}

fn parse_messages_of_content(locale_content: &str) -> Result<ModelMessages> {
    let mut lines_map = HashMap::new();
    let mut in_messages = false;

    for line in locale_content.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line == "LC_MESSAGES" {
            in_messages = true;
            continue;
        }

        if line == "END LC_MESSAGES" {
            break;
        }

        if !in_messages {
            continue;
        }

        let Some((name, value)) = line.split_once(char::is_whitespace) else {
            return Err(anyhow!("Malformed line: {line}"));
        };

        lines_map.insert(name, parse_value(value)?);
    }

    if !in_messages {
        return Err(anyhow!("Locale doesn't contain messages section"));
    }

    //let lines_map.remove = |name: &str| lines_map.remove(name);

    Ok(ModelMessages {
        yesexpr: lines_map.remove("yesexpr"),
        noexpr: lines_map.remove("noexpr"),
        e_: lines_map.remove("E_"),
        e0: lines_map.remove("E0"),
        eperm: lines_map.remove("EPERM"),
        enoent: lines_map.remove("ENOENT"),
        esrch: lines_map.remove("ESRCH"),
        eintr: lines_map.remove("EINTR"),
        eio: lines_map.remove("EIO"),
        enxio: lines_map.remove("ENXIO"),
        e2big: lines_map.remove("E2BIG"),
        enoexec: lines_map.remove("ENOEXEC"),
        ebadf: lines_map.remove("EBADF"),
        echild: lines_map.remove("ECHILD"),
        eagain: lines_map.remove("EAGAIN"),
        enomem: lines_map.remove("ENOMEM"),
        eacces: lines_map.remove("EACCES"),
        efault: lines_map.remove("EFAULT"),
        enotblk: lines_map.remove("ENOTBLK"),
        ebusy: lines_map.remove("EBUSY"),
        eexist: lines_map.remove("EEXIST"),
        exdev: lines_map.remove("EXDEV"),
        enodev: lines_map.remove("ENODEV"),
        enotdir: lines_map.remove("ENOTDIR"),
        eisdir: lines_map.remove("EISDIR"),
        einval: lines_map.remove("EINVAL"),
        enfile: lines_map.remove("ENFILE"),
        emfile: lines_map.remove("EMFILE"),
        enotty: lines_map.remove("ENOTTY"),
        etxtbsy: lines_map.remove("ETXTBSY"),
        efbig: lines_map.remove("EFBIG"),
        enospc: lines_map.remove("ENOSPC"),
        espipe: lines_map.remove("ESPIPE"),
        erofs: lines_map.remove("EROFS"),
        emlink: lines_map.remove("EMLINK"),
        epipe: lines_map.remove("EPIPE"),
        edom: lines_map.remove("EDOM"),
        erange: lines_map.remove("ERANGE"),
        edeadlk: lines_map.remove("EDEADLK"),
        enametoolong: lines_map.remove("ENAMETOOLONG"),
        enolck: lines_map.remove("ENOLCK"),
        enosys: lines_map.remove("ENOSYS"),
        enotempty: lines_map.remove("ENOTEMPTY"),
        eloop: lines_map.remove("ELOOP"),
        ewouldblock: lines_map.remove("EWOULDBLOCK"),
        enomsg: lines_map.remove("ENOMSG"),
        eidrm: lines_map.remove("EIDRM"),
        echrng: lines_map.remove("ECHRNG"),
        el2nsync: lines_map.remove("EL2NSYNC"),
        el3hlt: lines_map.remove("EL3HLT"),
        el3rst: lines_map.remove("EL3RST"),
        elnrng: lines_map.remove("ELNRNG"),
        eunatch: lines_map.remove("EUNATCH"),
        enocsi: lines_map.remove("ENOCSI"),
        el2hlt: lines_map.remove("EL2HLT"),
        ebade: lines_map.remove("EBADE"),
        ebadr: lines_map.remove("EBADR"),
        exfull: lines_map.remove("EXFULL"),
        enoano: lines_map.remove("ENOANO"),
        ebadrqc: lines_map.remove("EBADRQC"),
        ebadslt: lines_map.remove("EBADSLT"),
        edeadlock: lines_map.remove("EDEADLOCK"),
        ebfont: lines_map.remove("EBFONT"),
        enostr: lines_map.remove("ENOSTR"),
        enodata: lines_map.remove("ENODATA"),
        etime: lines_map.remove("ETIME"),
        enosr: lines_map.remove("ENOSR"),
        enonet: lines_map.remove("ENONET"),
        enopkg: lines_map.remove("ENOPKG"),
        eremote: lines_map.remove("EREMOTE"),
        enolink: lines_map.remove("ENOLINK"),
        eadv: lines_map.remove("EADV"),
        esrmnt: lines_map.remove("ESRMNT"),
        ecomm: lines_map.remove("ECOMM"),
        eproto: lines_map.remove("EPROTO"),
        emultihop: lines_map.remove("EMULTIHOP"),
        edotdot: lines_map.remove("EDOTDOT"),
        ebadmsg: lines_map.remove("EBADMSG"),
        eoverflow: lines_map.remove("EOVERFLOW"),
        enotuniq: lines_map.remove("ENOTUNIQ"),
        ebadfd: lines_map.remove("EBADFD"),
        eremchg: lines_map.remove("EREMCHG"),
        elibacc: lines_map.remove("ELIBACC"),
        elibbad: lines_map.remove("ELIBBAD"),
        elibscn: lines_map.remove("ELIBSCN"),
        elibmax: lines_map.remove("ELIBMAX"),
        elibexec: lines_map.remove("ELIBEXEC"),
        eilseq: lines_map.remove("EILSEQ"),
        erestart: lines_map.remove("ERESTART"),
        estrpipe: lines_map.remove("ESTRPIPE"),
        eusers: lines_map.remove("EUSERS"),
        enotsock: lines_map.remove("ENOTSOCK"),
        edestaddrreq: lines_map.remove("EDESTADDRREQ"),
        emsgsize: lines_map.remove("EMSGSIZE"),
        eprototype: lines_map.remove("EPROTOTYPE"),
        enoprotoopt: lines_map.remove("ENOPROTOOPT"),
        eprotonosupport: lines_map.remove("EPROTONOSUPPORT"),
        esocktnosupport: lines_map.remove("ESOCKTNOSUPPORT"),
        eopnotsupp: lines_map.remove("EOPNOTSUPP"),
        enotsup: lines_map.remove("ENOTSUP"),
        epfnosupport: lines_map.remove("EPFNOSUPPORT"),
        eafnosupport: lines_map.remove("EAFNOSUPPORT"),
        eaddrinuse: lines_map.remove("EADDRINUSE"),
        eaddrnotavail: lines_map.remove("EADDRNOTAVAIL"),
        enetdown: lines_map.remove("ENETDOWN"),
        enetunreach: lines_map.remove("ENETUNREACH"),
        enetreset: lines_map.remove("ENETRESET"),
        econnaborted: lines_map.remove("ECONNABORTED"),
        econnreset: lines_map.remove("ECONNRESET"),
        enobufs: lines_map.remove("ENOBUFS"),
        eisconn: lines_map.remove("EISCONN"),
        enotconn: lines_map.remove("ENOTCONN"),
        eshutdown: lines_map.remove("ESHUTDOWN"),
        etoomanyrefs: lines_map.remove("ETOOMANYREFS"),
        etimedout: lines_map.remove("ETIMEDOUT"),
        econnrefused: lines_map.remove("ECONNREFUSED"),
        ehostdown: lines_map.remove("EHOSTDOWN"),
        ehostunreach: lines_map.remove("EHOSTUNREACH"),
        ealready: lines_map.remove("EALREADY"),
        einprogress: lines_map.remove("EINPROGRESS"),
        estale: lines_map.remove("ESTALE"),
        euclean: lines_map.remove("EUCLEAN"),
        enotnam: lines_map.remove("ENOTNAM"),
        enavail: lines_map.remove("ENAVAIL"),
        eisnam: lines_map.remove("EISNAM"),
        eremoteio: lines_map.remove("EREMOTEIO"),
        edquot: lines_map.remove("EDQUOT"),
        enomedium: lines_map.remove("ENOMEDIUM"),
        emediumtype: lines_map.remove("EMEDIUMTYPE"),
        ecanceled: lines_map.remove("ECANCELED"),
        enokey: lines_map.remove("ENOKEY"),
        ekeyexpired: lines_map.remove("EKEYEXPIRED"),
        ekeyrevoked: lines_map.remove("EKEYREVOKED"),
        ekeyrejected: lines_map.remove("EKEYREJECTED"),
        eownerdead: lines_map.remove("EOWNERDEAD"),
        enotrecoverable: lines_map.remove("ENOTRECOVERABLE"),
        erfkill: lines_map.remove("ERFKILL"),
        ehwpoison: lines_map.remove("EHWPOISON"),
        reg__: lines_map.remove("REG__"),
        reg_ok: lines_map.remove("REG_OK"),
        reg_nomatch: lines_map.remove("REG_NOMATCH"),
        reg_badpat: lines_map.remove("REG_BADPAT"),
        reg_ecollate: lines_map.remove("REG_ECOLLATE"),
        reg_ectype: lines_map.remove("REG_ECTYPE"),
        reg_eescape: lines_map.remove("REG_EESCAPE"),
        reg_esubreg: lines_map.remove("REG_ESUBREG"),
        reg_ebrack: lines_map.remove("REG_EBRACK"),
        reg_eparen: lines_map.remove("REG_EPAREN"),
        reg_ebrace: lines_map.remove("REG_EBRACE"),
        reg_badbr: lines_map.remove("REG_BADBR"),
        reg_erange: lines_map.remove("REG_ERANGE"),
        reg_espace: lines_map.remove("REG_ESPACE"),
        reg_badrpt: lines_map.remove("REG_BADRPT"),
        eai__: lines_map.remove("EAI__"),
        eai_0: lines_map.remove("EAI_0"),
        eai_badflags: lines_map.remove("EAI_BADFLAGS"),
        eai_noname: lines_map.remove("EAI_NONAME"),
        eai_again: lines_map.remove("EAI_AGAIN"),
        eai_fail: lines_map.remove("EAI_FAIL"),
        eai_nodata: lines_map.remove("EAI_NODATA"),
        eai_family: lines_map.remove("EAI_FAMILY"),
        eai_socktype: lines_map.remove("EAI_SOCKTYPE"),
        eai_service: lines_map.remove("EAI_SERVICE"),
        eai_memory: lines_map.remove("EAI_MEMORY"),
        eai_system: lines_map.remove("EAI_SYSTEM"),
        eai_overflow: lines_map.remove("EAI_OVERFLOW"),
        h_: lines_map.remove("H_"),
        h0: lines_map.remove("H0"),
        host_not_found: lines_map.remove("HOST_NOT_FOUND"),
        try_again: lines_map.remove("TRY_AGAIN"),
        no_recovery: lines_map.remove("NO_RECOVERY"),
        no_data: lines_map.remove("NO_DATA"),
    })
}

/// Removes the quotes around a locale value and decodes the escapes used by
/// the locale definition format.
fn parse_value(value: &str) -> Result<String> {
    let value = value.trim();

    let Some(value) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return Err(anyhow!("Malformed value: {value}"));
    };

    let mut result = String::with_capacity(value.len());
    let mut characters = value.chars();

    while let Some(character) = characters.next() {
        if character != '\\' {
            result.push(character);
            continue;
        }

        match characters.next() {
            Some('n') => result.push('\n'),
            Some('t') => result.push('\t'),
            Some(escaped) => result.push(escaped),
            None => result.push('\\'),
        }
    }

    Ok(result)
}
