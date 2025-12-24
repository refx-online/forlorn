use std::{net::IpAddr, sync::LazyLock};

use axum::http::HeaderMap;

#[derive(Debug)]
pub struct Geolocation {
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// since .is_global() is unstable, we inversely check for private ips.
fn is_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_private()
                || ipv4.is_loopback()
                || ipv4.is_link_local()
                || ipv4.is_broadcast()
                || ipv4.is_documentation()
                || ipv4.is_unspecified()
        },
        IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unspecified() || ipv6.is_multicast(),
    }
}

fn get_ip_from_headers(headers: &HeaderMap) -> Option<IpAddr> {
    if let Some(ip) = headers.get("CF-Connecting-IP")
        && let Ok(ip_str) = ip.to_str()
        && let Ok(addr) = ip_str.parse::<IpAddr>()
    {
        return Some(addr);
    }

    if let Some(forwarded) = headers.get("X-Forwarded-For")
        && let Ok(forwarded_str) = forwarded.to_str()
    {
        let ips: Vec<&str> = forwarded_str.split(',').collect();

        if ips.len() > 1
            && let Ok(addr) = ips[0].trim().parse::<IpAddr>()
        {
            return Some(addr);
        }
    }

    if let Some(ip) = headers.get("X-Real-IP")
        && let Ok(ip_str) = ip.to_str()
        && let Ok(addr) = ip_str.parse::<IpAddr>()
    {
        return Some(addr);
    }

    None
}

fn fetch_geoloc_cloudflare(headers: &HeaderMap) -> Option<Geolocation> {
    let country = headers.get("CF-IPCountry")?.to_str().ok()?;
    let lat = headers.get("CF-IPLatitude")?.to_str().ok()?.parse().ok()?;
    let lon = headers.get("CF-IPLongitude")?.to_str().ok()?.parse().ok()?;

    Some(Geolocation {
        latitude: lat,
        longitude: lon,
        country_code: country.to_lowercase(),
    })
}

async fn fetch_geoloc_from_ip(ip: IpAddr) -> Option<Geolocation> {
    let url = if !is_private(ip) {
        format!("http://ip-api.com/line/{ip}")
    } else {
        "http://ip-api.com/line/".to_string()
    };

    let response = CLIENT
        .get(&url)
        .query(&[("fields", "status,message,countryCode,lat,lon")])
        .send()
        .await
        .ok()?;

    if response.status() != 200 {
        return None;
    }

    let text = response.text().await.ok()?;
    let lines: Vec<&str> = text.lines().collect();

    if lines.first()? != &"success" {
        return None;
    }

    Some(Geolocation {
        country_code: lines.get(1)?.to_lowercase(),
        latitude: lines.get(2)?.parse().ok()?,
        longitude: lines.get(3)?.parse().ok()?,
    })
}

pub async fn fetch_geoloc(headers: &HeaderMap) -> Option<Geolocation> {
    if let Some(geoloc) = fetch_geoloc_cloudflare(headers) {
        return Some(geoloc);
    }

    if let Some(ip) = get_ip_from_headers(headers) {
        return fetch_geoloc_from_ip(ip).await;
    }

    None
}
