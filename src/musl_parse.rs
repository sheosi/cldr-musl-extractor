use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Result, anyhow};

use crate::model::ModelMessages;

/// Returns the messages saved for a language, if the file doesn't exist, returns
/// the default en_US messages, if there's any other error it is returned.
pub async fn parse_messages(lang: &str, region: &str) -> Result<ModelMessages> {
    let path = PathBuf::from(format!("musl-locales/locales/{lang}_{region}"));
    if !path.exists() {
        return Ok(ModelMessages::default());
    };

    let file_contents = tokio::fs::read_to_string(path).await?;
    parse_messages_of_content(&file_contents)
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

    let get_msg = |name: &str| {
        lines_map
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Failed to get message {name}"))
    };

    Ok(ModelMessages {
        yesexpr: get_msg("yesexpr")?,
        noexpr: get_msg("noexpr")?,
        e_: get_msg("E_")?,
        e0: get_msg("E0")?,
        eperm: get_msg("EPERM")?,
        enoent: get_msg("ENOENT")?,
        esrch: get_msg("ESRCH")?,
        eintr: get_msg("EINTR")?,
        eio: get_msg("EIO")?,
        enxio: get_msg("ENXIO")?,
        e2big: get_msg("E2BIG")?,
        enoexec: get_msg("ENOEXEC")?,
        ebadf: get_msg("EBADF")?,
        echild: get_msg("ECHILD")?,
        eagain: get_msg("EAGAIN")?,
        enomem: get_msg("ENOMEM")?,
        eacces: get_msg("EACCES")?,
        efault: get_msg("EFAULT")?,
        enotblk: get_msg("ENOTBLK")?,
        ebusy: get_msg("EBUSY")?,
        eexist: get_msg("EEXIST")?,
        exdev: get_msg("EXDEV")?,
        enodev: get_msg("ENODEV")?,
        enotdir: get_msg("ENOTDIR")?,
        eisdir: get_msg("EISDIR")?,
        einval: get_msg("EINVAL")?,
        enfile: get_msg("ENFILE")?,
        emfile: get_msg("EMFILE")?,
        enotty: get_msg("ENOTTY")?,
        etxtbsy: get_msg("ETXTBSY")?,
        efbig: get_msg("EFBIG")?,
        enospc: get_msg("ENOSPC")?,
        espipe: get_msg("ESPIPE")?,
        erofs: get_msg("EROFS")?,
        emlink: get_msg("EMLINK")?,
        epipe: get_msg("EPIPE")?,
        edom: get_msg("EDOM")?,
        erange: get_msg("ERANGE")?,
        edeadlk: get_msg("EDEADLK")?,
        enametoolong: get_msg("ENAMETOOLONG")?,
        enolck: get_msg("ENOLCK")?,
        enosys: get_msg("ENOSYS")?,
        enotempty: get_msg("ENOTEMPTY")?,
        eloop: get_msg("ELOOP")?,
        ewouldblock: get_msg("EWOULDBLOCK")?,
        enomsg: get_msg("ENOMSG")?,
        eidrm: get_msg("EIDRM")?,
        echrng: get_msg("ECHRNG")?,
        el2nsync: get_msg("EL2NSYNC")?,
        el3hlt: get_msg("EL3HLT")?,
        el3rst: get_msg("EL3RST")?,
        elnrng: get_msg("ELNRNG")?,
        eunatch: get_msg("EUNATCH")?,
        enocsi: get_msg("ENOCSI")?,
        el2hlt: get_msg("EL2HLT")?,
        ebade: get_msg("EBADE")?,
        ebadr: get_msg("EBADR")?,
        exfull: get_msg("EXFULL")?,
        enoano: get_msg("ENOANO")?,
        ebadrqc: get_msg("EBADRQC")?,
        ebadslt: get_msg("EBADSLT")?,
        edeadlock: get_msg("EDEADLOCK")?,
        ebfont: get_msg("EBFONT")?,
        enostr: get_msg("ENOSTR")?,
        enodata: get_msg("ENODATA")?,
        etime: get_msg("ETIME")?,
        enosr: get_msg("ENOSR")?,
        enonet: get_msg("ENONET")?,
        enopkg: get_msg("ENOPKG")?,
        eremote: get_msg("EREMOTE")?,
        enolink: get_msg("ENOLINK")?,
        eadv: get_msg("EADV")?,
        esrmnt: get_msg("ESRMNT")?,
        ecomm: get_msg("ECOMM")?,
        eproto: get_msg("EPROTO")?,
        emultihop: get_msg("EMULTIHOP")?,
        edotdot: get_msg("EDOTDOT")?,
        ebadmsg: get_msg("EBADMSG")?,
        eoverflow: get_msg("EOVERFLOW")?,
        enotuniq: get_msg("ENOTUNIQ")?,
        ebadfd: get_msg("EBADFD")?,
        eremchg: get_msg("EREMCHG")?,
        elibacc: get_msg("ELIBACC")?,
        elibbad: get_msg("ELIBBAD")?,
        elibscn: get_msg("ELIBSCN")?,
        elibmax: get_msg("ELIBMAX")?,
        elibexec: get_msg("ELIBEXEC")?,
        eilseq: get_msg("EILSEQ")?,
        erestart: get_msg("ERESTART")?,
        estrpipe: get_msg("ESTRPIPE")?,
        eusers: get_msg("EUSERS")?,
        enotsock: get_msg("ENOTSOCK")?,
        edestaddrreq: get_msg("EDESTADDRREQ")?,
        emsgsize: get_msg("EMSGSIZE")?,
        eprototype: get_msg("EPROTOTYPE")?,
        enoprotoopt: get_msg("ENOPROTOOPT")?,
        eprotonosupport: get_msg("EPROTONOSUPPORT")?,
        esocktnosupport: get_msg("ESOCKTNOSUPPORT")?,
        eopnotsupp: get_msg("EOPNOTSUPP")?,
        enotsup: get_msg("ENOTSUP")?,
        epfnosupport: get_msg("EPFNOSUPPORT")?,
        eafnosupport: get_msg("EAFNOSUPPORT")?,
        eaddrinuse: get_msg("EADDRINUSE")?,
        eaddrnotavail: get_msg("EADDRNOTAVAIL")?,
        enetdown: get_msg("ENETDOWN")?,
        enetunreach: get_msg("ENETUNREACH")?,
        enetreset: get_msg("ENETRESET")?,
        econnaborted: get_msg("ECONNABORTED")?,
        econnreset: get_msg("ECONNRESET")?,
        enobufs: get_msg("ENOBUFS")?,
        eisconn: get_msg("EISCONN")?,
        enotconn: get_msg("ENOTCONN")?,
        eshutdown: get_msg("ESHUTDOWN")?,
        etoomanyrefs: get_msg("ETOOMANYREFS")?,
        etimedout: get_msg("ETIMEDOUT")?,
        econnrefused: get_msg("ECONNREFUSED")?,
        ehostdown: get_msg("EHOSTDOWN")?,
        ehostunreach: get_msg("EHOSTUNREACH")?,
        ealready: get_msg("EALREADY")?,
        einprogress: get_msg("EINPROGRESS")?,
        estale: get_msg("ESTALE")?,
        euclean: get_msg("EUCLEAN")?,
        enotnam: get_msg("ENOTNAM")?,
        enavail: get_msg("ENAVAIL")?,
        eisnam: get_msg("EISNAM")?,
        eremoteio: get_msg("EREMOTEIO")?,
        edquot: get_msg("EDQUOT")?,
        enomedium: get_msg("ENOMEDIUM")?,
        emediumtype: get_msg("EMEDIUMTYPE")?,
        ecanceled: get_msg("ECANCELED")?,
        enokey: get_msg("ENOKEY")?,
        ekeyexpired: get_msg("EKEYEXPIRED")?,
        ekeyrevoked: get_msg("EKEYREVOKED")?,
        ekeyrejected: get_msg("EKEYREJECTED")?,
        eownerdead: get_msg("EOWNERDEAD")?,
        enotrecoverable: get_msg("ENOTRECOVERABLE")?,
        erfkill: get_msg("ERFKILL")?,
        ehwpoison: get_msg("EHWPOISON")?,
        reg__: get_msg("REG__")?,
        reg_ok: get_msg("REG_OK")?,
        reg_nomatch: get_msg("REG_NOMATCH")?,
        reg_badpat: get_msg("REG_BADPAT")?,
        reg_ecollate: get_msg("REG_ECOLLATE")?,
        reg_ectype: get_msg("REG_ECTYPE")?,
        reg_eescape: get_msg("REG_EESCAPE")?,
        reg_esubreg: get_msg("REG_ESUBREG")?,
        reg_ebrack: get_msg("REG_EBRACK")?,
        reg_eparen: get_msg("REG_EPAREN")?,
        reg_ebrace: get_msg("REG_EBRACE")?,
        reg_badbr: get_msg("REG_BADBR")?,
        reg_erange: get_msg("REG_ERANGE")?,
        reg_espace: get_msg("REG_ESPACE")?,
        reg_badrpt: get_msg("REG_BADRPT")?,
        eai__: get_msg("EAI__")?,
        eai_0: get_msg("EAI_0")?,
        eai_badflags: get_msg("EAI_BADFLAGS")?,
        eai_noname: get_msg("EAI_NONAME")?,
        eai_again: get_msg("EAI_AGAIN")?,
        eai_fail: get_msg("EAI_FAIL")?,
        eai_nodata: get_msg("EAI_NODATA")?,
        eai_family: get_msg("EAI_FAMILY")?,
        eai_socktype: get_msg("EAI_SOCKTYPE")?,
        eai_service: get_msg("EAI_SERVICE")?,
        eai_memory: get_msg("EAI_MEMORY")?,
        eai_system: get_msg("EAI_SYSTEM")?,
        eai_overflow: get_msg("EAI_OVERFLOW")?,
        h_: get_msg("H_")?,
        h0: get_msg("H0")?,
        host_not_found: get_msg("HOST_NOT_FOUND")?,
        try_again: get_msg("TRY_AGAIN")?,
        no_recovery: get_msg("NO_RECOVERY")?,
        no_data: get_msg("NO_DATA")?,
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
