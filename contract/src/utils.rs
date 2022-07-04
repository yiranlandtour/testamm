use near_sdk::serde::Deserialize;
use near_sdk::PromiseResult;

pub fn parse_promise_result<'de, T>(rslt: &'de PromiseResult) -> Option<T>
where
    T: Deserialize<'de>,
{
    let data = match rslt {
        PromiseResult::NotReady => unreachable!(),
        PromiseResult::Successful(val) => {
            if let Ok(metadata) = near_sdk::serde_json::from_slice::<T>(&val) {
                Some(metadata)
            } else {
                None 
            }
        }
        PromiseResult::Failed => None,
    };

    return data;
}
