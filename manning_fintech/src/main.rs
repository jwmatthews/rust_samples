use std::collections::HashMap;
use std::io;


/// An application-specific error type
#[derive(Debug)]
enum AccountingError {
    // Add variants here for account not found, account underfunded and account overfunded
    AccountNotFound(String),
    AccountUnderFunded(String, u64),
    AccountOverFunded(String, u64),
}

/// A transaction type. Transactions should be able to rebuild a ledger's state
/// when they are applied in the same sequence to an empty state.
#[derive(Debug)]
pub enum Tx {
    // Add variants for storing withdraw/deposit transactions
    Deposit { account: String, amount: u64 },
    Withdraw { account: String, amount: u64 },
}

/// A type for managing accounts and their current currency balance
#[derive(Debug)]
struct Accounts {
    accounts: HashMap<String, u64>,
}

impl Accounts {
    /// Returns an empty instance of the [`Accounts`] type
    pub fn new() -> Self {
        Accounts {
            accounts: Default::default(),
        }
    }

    /// Either deposits the `amount` provided into the `signer` account or adds the amount to the existing account.
    /// # Errors
    /// Attempted overflow
    pub fn deposit(&mut self, signer: &str, amount: u64) -> Result<Tx, AccountingError> {
        if let Some(account) = self.accounts.get_mut(signer) {
            //JWM:  I am confused why this was called 'account', it makes me think more of an 'id',
            // yet this is really the balance in the account.
            (*account)
                .checked_add(amount)
                .and_then(|r| {
                    *account = r;
                    Some(r)
                })
                .ok_or(AccountingError::AccountOverFunded(
                    signer.to_string(),
                    amount,
                ))
                // Using map() here is an easy way to only manipulate the non-error result
                .map(|_| Tx::Deposit {
                    account: signer.to_string(),
                    amount,
                })
        } else {
            self.accounts.insert(signer.to_string(), amount);
            Ok(Tx::Deposit {
                account: signer.to_string(),
                amount,
            })
        }
    }

    /// Withdraws the `amount` from the `signer` account.
    /// # Errors
    /// Attempted overflow
    pub fn withdraw(&mut self, signer: &str, amount: u64) -> Result<Tx, AccountingError> {
        if let Some(balance) = self.accounts.get_mut(signer) {
            (*balance)
                .checked_sub(amount)
                .and_then(|r| {
                    *balance = r;
                    Some(r)
                })
                .ok_or(AccountingError::AccountUnderFunded(
                    signer.to_string(),
                    amount,
                ))
                // Using map() here is an easy way to only manipulate the non-error result
                .map(|_| Tx::Withdraw {
                    account: signer.to_string(),
                    amount,
                })
        } else {
            Err(AccountingError::AccountNotFound(signer.to_string()))
        }
    }

    /// Withdraws the amount from the sender account and deposits it in the recipient account.
    ///
    /// # Errors
    /// The account doesn't exist
    pub fn send_version1(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<(Tx, Tx), AccountingError> {
        // Concerns:
        // 1) Need to ensure sender and recipient exist
        //     error of AccountingError::AccountNotFound
        // 2) Ensure sender has funds to cover withdrawal
        //     error of AccountingError::UnderFunded
        // 3) Ensure recipient can received and doesn't overflow
        //     error of AccountingError:OverFunded
        //     *if* an error occurs here we don't want to lose money from senders account
        //
        match self.withdraw(sender, amount) {
            Ok(w_tx) => {
                match self.deposit(recipient, amount) {
                    Ok(d_tx) => {
                        // Success, return both transactions
                        Ok((w_tx, d_tx))
                    }
                    Err(d_err) => {
                        // We saw an error from depositing to recipient,
                        // yet we already deducted from sender, let's reverse the deduction
                        match self.deposit(sender, amount) {
                            Ok(_) => Err(d_err),
                            Err(e) => {
                                // Not ideal, we are swallowing the error from deposit to recipient
                                // and just sending back the error from deposit to sender
                                // Not ideal, unsure best thing for this case without making larger changes
                                Err(e)
                            }
                        }
                    }
                }
            }
            Err(w_err) => {
                // Returning the error we received from 'withdraw'
                Err(w_err)
            }
        }
    }

    /// Withdraws the amount from the sender account and deposits it in the recipient account.
    ///
    /// # Errors
    /// The account doesn't exist
    pub fn send(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<(Tx, Tx), AccountingError> {
        // Concerns:
        // 1) Need to ensure sender and recipient exist
        //     error of AccountingError::AccountNotFound
        // 2) Ensure sender has funds to cover withdrawal
        //     error of AccountingError::UnderFunded
        // 3) Ensure recipient can received and doesn't overflow
        //     error of AccountingError:OverFunded
        //     *if* an error occurs here we don't want to lose money from senders account
        //
        if self.accounts.contains_key(sender)  // sender exists
            && self.accounts.contains_key(recipient) // recipient exists
            && self
            .accounts
            .get(sender)
            .map(|amt| *amt >= amount) // sender has sufficient funds
            .unwrap_or(false)
        {
            // The ? operator is a built-in shorthand for
            // if let Err(e) = my_func_call() { return Err(e); }
            let tx_withdraw = self.withdraw(sender, amount)?;
            self.deposit(recipient, amount)
                .map_err(|e| {
                    // return the funds to the sender on error
                    self.deposit(sender, amount).unwrap();
                    e
                })
                .map(|tx_deposit| (tx_withdraw, tx_deposit))
        } else {
            if !self.accounts.contains_key(sender) {
                Err(AccountingError::AccountNotFound(sender.to_string()))
            } else if !self.accounts.contains_key(recipient) {
                Err(AccountingError::AccountNotFound(recipient.to_string()))
            } else {
                // Case for error on underfunded was missing, would have
                // been reported as a missing 'recipient'.
                Err(AccountingError::AccountUnderFunded(sender.to_string(), amount))
            }
        }
    }
}


fn read_from_stdin(label: &str) -> String {
    // This function should print the label to the command line.
    // After that, wait for user input that is terminated by a newline and return it.
    // Use trim() to make sure the input is sanitized.
    let mut user_input = String::new();
    let stdin = io::stdin();

    println!("{}", label);
    stdin.read_line(&mut user_input).expect("Problem reading from stdin");
    user_input.trim().to_string()
}

fn main() {

    let mut ledger = Accounts::new();

    // deposit, withdraw, send, print, quit
    loop {
        let command = read_from_stdin("Input: ");
        match command {
            "deposit" => _,
            "withdraw" => _,
            "send" => _,
            "print" => _,
            "quit" => _,
            _ => println!("Didn't recognize '{}' as a known command", command);
        }
    }

}

fn main_old() {
    println!("Hello, accounting world!");

    // We are using simple &str instances as keys
    // for more sophisticated keys (e.g. hashes)
    // the data type could remain the same
    let bob = "bob";
    let alice = "alice";
    let charlie = "charlie";
    let initial_amount = 100;

    // Creates the basic ledger and a tx log container
    let mut ledger = Accounts::new();
    let mut tx_log = vec![];

    // Deposit an amount to each account
    for signer in &[bob, alice, charlie] {
        let status = ledger.deposit(*signer, initial_amount);
        println!("Depositing {} for {}: {:?}", signer, initial_amount, status);
        // Add the resulting transaction to a list of transactions
        // .unwrap() will crash the program if the status is an error.
        tx_log.push(status.unwrap());
    }

    // Send currency from one account (bob) to the other (alice)
    let send_amount = 10_u64;
    let status = ledger.send(bob, alice, send_amount);
    println!(
        "Sent {} from {} to {}: {:?}",
        send_amount, bob, alice, status
    );

    // Add both transactions to the transaction log
    let (tx1, tx2) = status.unwrap();
    tx_log.push(tx1);
    tx_log.push(tx2);

    // Withdraw everything from the accounts
    let tx = ledger.withdraw(charlie, initial_amount).unwrap();
    tx_log.push(tx);
    let tx = ledger
        .withdraw(alice, initial_amount + send_amount)
        .unwrap();
    tx_log.push(tx);

    // Here we are withdrawing too much and there won't be a transaction
    println!(
        "Withdrawing {} from {}: {:?}",
        initial_amount,
        bob,
        ledger.withdraw(bob, initial_amount)
    );
    // Withdrawing the expected amount results in a transaction
    let tx = ledger.withdraw(bob, initial_amount - send_amount).unwrap();
    tx_log.push(tx);

    // {:?} prints the Debug implementation, {:#?} pretty-prints it
    println!("Ledger empty: {:?}", ledger);
    println!("The TX log: {:#?}", tx_log);
}
