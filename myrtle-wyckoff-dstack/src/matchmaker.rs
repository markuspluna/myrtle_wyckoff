extern crate optimized_lob;
use alloy::primitives::U256;
use optimized_lob::{
    order::OrderId, orderbook_manager::OrderBookManager, price::Price, quantity::Qty, utils::BookId,
};
/// Overview: matching engine
/// Matches an order against the current orderbook state, performing executions where necessary.
///
/// Returns a tuple of 4 values:
/// - total qty executed
/// - total volume of matches (this is the revenue for an ask and the cost for a bid)
/// - new order id if one was created
/// - vec of filled order tuples (oid,price) (if any) - we do not include qty here to remove additional read requirements
/// - partially filled order tuple (oid,price) (if any)
///
/// Notes:
/// * Attempts the following process:
///   1. Receive order from user
///   2. Check bids or asks and price levels to see what levels if any it crosses
///   3. Check levels to get quantities to see how many get cleared
///   4. Remove orders from all crossed levels and reduce from unfilled levels
///   5. Add order for any remaining size
///
/// * This is the expected entrypoint for all new user trades
/// * This isn't necessarily a standard matching engine implementation, it's just simple
pub fn match_order(
    manager: &mut OrderBookManager,
    book_id: BookId,
    price256: U256,
    qty: Qty,
    is_bid: bool,
) -> (
    Qty,
    Qty,
    Option<OrderId>,
    Vec<(OrderId, Price)>,
    Option<(OrderId, Price)>,
) {
    let mut filled_orders: Vec<(OrderId, Price)> = Vec::new();
    let mut partially_filled_order_id: Option<(OrderId, Price)> = None;
    let mut remaining_qty = qty;
    let mut volume = Qty(U256::ZERO);
    let mut new_order_id = None;
    // flip the order type to match the price with the opposite types below
    let match_price = Price::from_u256(price256, !is_bid);

    if let Some(book) = manager.books.get_mut(book_id.value() as usize).unwrap() {
        let levels = if is_bid {
            &mut book.asks
        } else {
            &mut book.bids
        };

        // Get all crossed levels
        let mut crossed_levels = Vec::new();

        let mut i = levels.len();
        while i > 0 {
            i -= 1;
            let level = levels.get(i);

            // works for both bids and asks
            if level.price() < match_price {
                break; // No more matching levels
            }
            crossed_levels.push(level.level_id());
        }
        let level_pool = book.level_pool.clone();
        let level_orders = book.level_orders.clone();
        // Execute orders on crossed levels
        for level_id in crossed_levels.iter() {
            let level = level_pool.get(*level_id).unwrap();
            let level_size = level.size();
            let level_price = level.price().absolute();
            let order_iter = level_orders.get(level_id).unwrap().iter();

            match level_size.cmp(&remaining_qty) {
                std::cmp::Ordering::Less => {
                    for order in order_iter {
                        manager.execute_order(*order, remaining_qty);
                        filled_orders.push((*order, level.price()));
                    }
                    remaining_qty -= level_size;
                    volume += Qty(level_size.value() * level_price);
                }
                std::cmp::Ordering::Greater => {
                    volume += Qty(remaining_qty.value() * level_price);
                    for order in order_iter {
                        let old_order = manager.oid_map.get(*order).unwrap().clone();
                        manager.execute_order(*order, remaining_qty);
                        match old_order.qty().cmp(&remaining_qty) {
                            std::cmp::Ordering::Less => {
                                remaining_qty -= old_order.qty();
                                filled_orders.push((*order, level.price()));
                            }
                            std::cmp::Ordering::Equal => {
                                filled_orders.push((*order, level.price()));
                                remaining_qty = Qty(U256::ZERO);
                                break;
                            }
                            std::cmp::Ordering::Greater => {
                                partially_filled_order_id = Some((*order, level.price()));
                                remaining_qty = Qty(U256::ZERO);
                                break;
                            }
                        }
                    }
                }
                std::cmp::Ordering::Equal => {
                    for order in order_iter {
                        manager.execute_order(*order, remaining_qty);
                        filled_orders.push((*order, level.price()));
                    }
                    remaining_qty = Qty(U256::ZERO);
                    volume += Qty(level_size.value() * level_price);
                    break;
                }
            }
        }
    }

    if remaining_qty.gt(&Qty(U256::ZERO)) {
        let order_id = manager.oid_map.next_id();
        manager.add_order(order_id, book_id, remaining_qty, price256, is_bid);
        new_order_id = Some(order_id);
    }
    (
        volume,
        Qty(qty.value() - remaining_qty.value()),
        new_order_id,
        filled_orders,
        partially_filled_order_id,
    )
}
