include!("host_target_locator/preamble.rs");
include!("host_target_locator/types.rs");
include!("host_target_locator/logic.rs");

#[cfg(test)]
mod tests {
    include!("host_target_locator/tests.rs");
}
